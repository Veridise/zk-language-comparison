use halo2_base::gates::{GateChip, RangeChip, RangeInstructions};
use halo2_base::poseidon::hasher::PoseidonHasher;
use halo2_base::utils::BigPrimeField;
use halo2_base::{AssignedValue, Context};
use snark_verifier_sdk::halo2::OptimizedPoseidonSpec;

const T: usize = 3;
const RATE: usize = 2;
const R_F: usize = 8;
const R_P: usize = 57;

pub fn hash_pegs<F: BigPrimeField>(
    ctx: &mut Context<F>,
    nonce: AssignedValue<F>,
    pegs: [AssignedValue<F>; 4],
) -> AssignedValue<F> {
    let poseidon_gate = GateChip::<F>::default();
    let mut poseidon =
        PoseidonHasher::<F, T, RATE>::new(OptimizedPoseidonSpec::new::<R_F, R_P, 0>());
    poseidon.initialize_consts(ctx, &poseidon_gate);
    let hash_inputs: [AssignedValue<F>; 5] = [nonce, pegs[0], pegs[1], pegs[2], pegs[3]];
    poseidon.hash_fix_len_array(ctx, &poseidon_gate, &hash_inputs)
}

const MAX_COLOR: u64 = 5;

pub fn assert_pegs_in_range<F: BigPrimeField>(
    range_chip: &RangeChip<F>,
    ctx: &mut Context<F>,
    pegs: [AssignedValue<F>; 4]) {

    pegs.iter().for_each(|v| {
        range_chip.check_less_than_safe(ctx, *v, MAX_COLOR + 1);
    });
}