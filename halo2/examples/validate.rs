use clap::Parser;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use halo2_base::gates::{GateChip, GateInstructions, RangeInstructions};
use halo2_base::halo2_proofs::dev::metadata::Gate;
use halo2_base::utils::{BigPrimeField, ScalarField};
use halo2_base::AssignedValue;
#[allow(unused_imports)]
use halo2_base::{
    Context,
    QuantumCell::{Constant, Existing, Witness},
};
use halo2_scaffold::scaffold::cmd::Cli;
use halo2_scaffold::scaffold::run;
use serde::{Deserialize, Serialize};

mod common;
use common::hash_pegs;
use snark_verifier_sdk::snark_verifier::loader::halo2::IntegerInstructions;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    nonce: String, // field element, but easier to deserialize as a string
    pegs: [String; 4],
    pub hash: String,
    pub guess: [String; 4],
}

fn codebreaker_validate<F: BigPrimeField>(
    builder: &mut BaseCircuitBuilder<F>,
    input: CircuitInput,
    make_public: &mut Vec<AssignedValue<F>>,
) {
    let ctx = builder.main(0);
    let nonce =
        ctx.load_witness(F::from_str_vartime(&input.nonce).expect("Error deserializing nonce"));
    let pegs: [AssignedValue<F>; 4] = input
        .pegs
        .map(|p| ctx.load_witness(F::from_str_vartime(&p).expect("Error deserializing peg")));
    let hash =
        ctx.load_witness(F::from_str_vartime(&input.hash).expect("Error deserializing hash"));
    let guesses: [AssignedValue<F>; 4] = input
        .guess
        .map(|g| ctx.load_witness(F::from_str_vartime(&g).expect("Error deserializing peg")));

    // Constrain that the correct game information has been loaded
    let hash_calc = hash_pegs(ctx, nonce, pegs);
    ctx.constrain_equal(&hash_calc, &hash);

    // Tally the correct guesses
    let equalities: Vec<AssignedValue<F>> = pegs
        .iter()
        .zip(guesses)
        .map(|(peg, guess)| -> AssignedValue<F> {
            let guess_chip = GateChip::<F>::default();
            guess_chip.is_equal(ctx, *peg, guess)
        })
        .collect();
    let correct_guesses = GateChip::<F>::default().sum(ctx, equalities);

    // Tally the number of partial guesses
    let partial_equalities: Vec<AssignedValue<F>> = guesses
        .iter()
        .enumerate()
        .map(|(i, guess)| {
            let partial_eqs: Vec<AssignedValue<F>> = (0..4)
                //.filter(|j| &i != j)
                .map(|j| GateChip::<F>::default().is_equal(ctx, *guess, pegs[j]))
                .collect();

            let partial_sum = GateChip::<F>::default().sum(ctx, partial_eqs);
            let partial_sum_is_zero = GateChip::<F>::default().is_zero(ctx, partial_sum);
            GateChip::<F>::default().not(ctx, partial_sum_is_zero)
        })
        .collect();
    let partial_guess_total = GateChip::<F>::default().sum(ctx, partial_equalities);
    let partial_guesses = GateInstructions::sub(
        &GateChip::<F>::default(),
        ctx,
        partial_guess_total,
        correct_guesses,
    );

    // Make the values public
    make_public.push(correct_guesses);
    make_public.push(partial_guesses);

    println!("Correct guesses: {:?}", correct_guesses.value());
    println!("Partial guesses: {:?}", partial_guesses.value());
}

fn main() {
    env_logger::init();
    let args = Cli::parse();
    run(codebreaker_validate, args);
}
