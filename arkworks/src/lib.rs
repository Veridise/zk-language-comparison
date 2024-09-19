use ark_bn254::Fr;
use ark_crypto_primitives::sponge::poseidon::constraints::*;
use ark_crypto_primitives::sponge::{constraints::CryptographicSpongeVar, poseidon::*};
use ark_ff::fields::PrimeField;
use ark_r1cs_std::cmp::*;
use ark_r1cs_std::prelude::*;
use ark_r1cs_std::uint64::UInt64;
use ark_relations::r1cs::{ConstraintSynthesizer, ConstraintSystemRef, SynthesisError};

/**
 * Create a PoseidonConfig. For our purposes, the exact config doesn't matter
 * too much, so long as it produces a valid hash.
 */
pub fn get_poseidon_config() -> PoseidonConfig<Fr> {
    // (rate, alpha, full_rounds, partial_rounds, skip_matrices)
    // PoseidonDefaultConfigEntry::new(2, 17, 8, 31, 0),
    // Adapted from https://github.com/PayneJoe/PNova/blob/main/src/provider/bn254.rs#L2
    let rate = 2;
    let alpha = 17;
    let full_rounds = 8;
    let partial_rounds = 31;
    let skip_matrices = 0;
    let capacity = 1;

    let (ark, mds) = find_poseidon_ark_and_mds::<Fr>(
        Fr::MODULUS_BIT_SIZE as u64,
        rate,
        full_rounds as u64,
        partial_rounds as u64,
        skip_matrices as u64,
    );
    let config = PoseidonConfig::<Fr> {
        full_rounds,
        partial_rounds,
        alpha,
        ark,
        mds,
        rate,
        capacity,
    };
    config
}

/**
 * Define the mastermind circuit.
 *
 * NPEGS: Number of peg colors (use 6 for a standard game)
 * SZ: Size of the code (use 4 for a standard game)
 */
#[derive(Clone)]
pub struct MastermindCircuit<const NPEGS: usize, const SZ: usize> {
    // Codemaker infomation:
    // - codemaker code, which must be private
    pub code: [Option<u64>; SZ],
    // - codemaker's nonce; private, used to protect against dictionary attacks against the code
    pub nonce: Option<u64>,

    // - codemaker's hash; public, identifies the current game
    pub hash: Option<u64>,
    // - codemaker's response; public
    pub num_partial_correct: Option<u64>,
    pub num_fully_correct: Option<u64>,

    // Codebreaker input (public):
    // - codebreaker guess (same size as the code)
    pub guess: [Option<u64>; SZ],
}

/**
 * Asserts that the pegs are legal by ensuring they are within the NPEGS bound:
 * in other words, check if each peg is a valid "color".
 */
fn assert_pegs_are_legal<const NPEGS: usize>(pegs: &Vec<UInt64<Fr>>) -> Result<(), SynthesisError> {
    let npegs_const = UInt64::<Fr>::constant(NPEGS.try_into().unwrap());
    let zero = UInt64::<Fr>::constant(0);

    for peg in pegs {
        let upper = peg.is_lt(&npegs_const)?;
        let lower = zero.is_le(peg)?;

        upper.enforce_equal(&Boolean::<Fr>::TRUE)?;
        lower.enforce_equal(&Boolean::<Fr>::TRUE)?;
    }

    Ok(())
}

/**
 * Assert that the given code is valid. This checks that the pegs are legal and
 * that the provided hash is equal to the hash of the nonce and code pegs.
 */
fn assert_code_is_valid<const NPEGS: usize>(
    cs: ConstraintSystemRef<Fr>,
    code: &Vec<UInt64<Fr>>,
    nonce: &UInt64<Fr>,
    hash: &UInt64<Fr>,
) -> Result<(), SynthesisError> {
    // The code must be a valid assignment.
    assert_pegs_are_legal::<NPEGS>(code)?;

    let config = get_poseidon_config();
    let mut sponge = PoseidonSpongeVar::<Fr>::new(cs, &config);

    let mut sponge_vals = vec![nonce.to_fp()?];
    for peg in code {
        sponge_vals.push(peg.to_fp()?);
    }
    sponge.absorb(&sponge_vals)?;

    let computed_hash = sponge.squeeze_field_elements(1)?;
    assert_eq!(computed_hash.len(), 1);
    let hash_bits = computed_hash[0].to_bits_le()?;
    let computed_hash_64 = UInt64::<Fr>::from_bits_le(&hash_bits[0..64]);

    computed_hash_64.enforce_equal(hash)?;

    Ok(())
}

/**
 * Compute the number of correct guesses.
 * The number of fully correct guesses ("black pegs") is:
 *      sum code[i] == guess[i] for i in range 0..number of pegs
 */
fn count_correct_guesses(
    code: &Vec<UInt64<Fr>>,
    guess: &Vec<UInt64<Fr>>,
) -> Result<UInt64<Fr>, SynthesisError> {
    let mut sum_fully_correct = UInt64::<Fr>::constant(0);
    for i in 0..code.len() {
        let is_correct = code[i].is_eq(&guess[i])?;
        // let is_correct_64 = UInt64::<Fr>::from_bits_le(&vec![is_correct]);
        let is_correct_64 =
            is_correct.select(&UInt64::<Fr>::constant(1), &UInt64::<Fr>::constant(0))?;
        sum_fully_correct.wrapping_add_in_place(&is_correct_64);
    }
    Ok(sum_fully_correct)
}

/**
 * Count the number of pegs of the given color present in the code.
 */
fn count_color(peg: &UInt64<Fr>, code: &Vec<UInt64<Fr>>) -> Result<UInt64<Fr>, SynthesisError> {
    let mut count = UInt64::<Fr>::constant(0);
    for i in 0..code.len() {
        let is_same_color = code[i].is_eq(peg)?;
        let is_same_color_64 =
            is_same_color.select(&UInt64::<Fr>::constant(1), &UInt64::<Fr>::constant(0))?;
        count.wrapping_add_in_place(&is_same_color_64);
    }
    Ok(count)
}

/**
 * Find the minimum between a and b.
 */
fn min(a: &UInt64<Fr>, b: &UInt64<Fr>) -> Result<UInt64<Fr>, SynthesisError> {
    a.is_lt(b)?.select(a, b)
}

/**
 * Compute the number of partially correct guesses.
 * The number of partially correct guesses ("white pegs") is:
 *      (sum min(count(i, code), count(i, guess)) i in range 0..peg colors) - correct pegs
 * In essence, for each color, count the number of pegs of that color in the guess and code. Find the minimum of those two numbers.
 * Then add this up across all colors, subtracting the overlap that are fully correct pegs.
 */
fn count_partial_guesses<const NPEGS: usize>(
    code: &Vec<UInt64<Fr>>,
    guess: &Vec<UInt64<Fr>>,
    fully_correct: &UInt64<Fr>,
) -> Result<UInt64<Fr>, SynthesisError> {
    let mut partial_sum = UInt64::<Fr>::constant(0);
    for p in 0..NPEGS {
        let peg = UInt64::<Fr>::constant(p as u64);
        let guess_count = count_color(&peg, guess)?;
        let code_count = count_color(&peg, code)?;
        let color_min = min(&guess_count, &code_count)?;
        partial_sum.wrapping_add_in_place(&color_min);
    }
    let fp_diff = partial_sum.to_fp()? - fully_correct.to_fp()?;
    let diff = UInt64::<Fr>::from_fp(&fp_diff)?.0;
    return Ok(diff);
}

/**
 * Assert that the response is valid, assuming that the code and guess are already
 * valid. This checks that, given the code and the guess, that the provided feedback
 * (number of black and white pegs, i.e. number of fully correct and partially correct
 * guesses) is correct.
 */
fn assert_response_is_valid<const NPEGS: usize>(
    code: &Vec<UInt64<Fr>>,
    guess: &Vec<UInt64<Fr>>,
    num_partial_correct: &UInt64<Fr>,
    num_fully_correct: &UInt64<Fr>,
) -> Result<(), SynthesisError> {
    // Assume we've already checked the code and guess for legality.
    let sum_fully_correct = count_correct_guesses(code, guess)?;
    sum_fully_correct.enforce_equal(num_fully_correct)?;

    let computed_partial_correct = count_partial_guesses::<NPEGS>(code, guess, &num_fully_correct)?;
    computed_partial_correct.enforce_equal(num_partial_correct)?;

    Ok(())
}

/**
 * This is the core logic of the mastermind circuit.
 */
impl<const NPEGS: usize, const SZ: usize> ConstraintSynthesizer<Fr>
    for MastermindCircuit<NPEGS, SZ>
{
    fn generate_constraints(self, cs: ConstraintSystemRef<Fr>) -> Result<(), SynthesisError> {
        // Allocate the variables
        // - private inputs are created via "new witness"
        let uint64_input_private = |opt: &Option<u64>| {
            UInt64::<Fr>::new_witness(cs.clone(), || opt.ok_or(SynthesisError::AssignmentMissing))
        };
        let uint64_arr_input_private = |arr: &[Option<u64>; SZ]| {
            arr.iter()
                .map(uint64_input_private)
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<_, _>>()
        };
        // - public inputs are created via "new input"
        let uint64_input_public = |opt: &Option<u64>| {
            UInt64::<Fr>::new_input(cs.clone(), || opt.ok_or(SynthesisError::AssignmentMissing))
        };
        let uint64_arr_input_public = |arr: &[Option<u64>; SZ]| {
            arr.iter()
                .map(uint64_input_public)
                .collect::<Vec<Result<_, _>>>()
                .into_iter()
                .collect::<Result<_, _>>()
        };

        // - Game info
        let code: Vec<_> = uint64_arr_input_private(&self.code)?;
        let nonce = uint64_input_private(&self.nonce)?;
        let hash = uint64_input_public(&self.hash)?;
        // - Codemaker response
        let num_partial_correct = uint64_input_public(&self.num_partial_correct)?;
        let num_fully_correct = uint64_input_public(&self.num_fully_correct)?;
        // - Codebreaker input
        let guess: Vec<_> = uint64_arr_input_public(&self.guess)?;

        // Make sure the code is valid
        assert_code_is_valid::<NPEGS>(cs, &code, &nonce, &hash)?;
        // Check that the guess is valid
        assert_pegs_are_legal::<NPEGS>(&guess)?;
        // Check that the response is valid
        assert_response_is_valid::<NPEGS>(&code, &guess, &num_partial_correct, &num_fully_correct)?;
        Ok(())
    }
}
