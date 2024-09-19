use clap::Parser;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use halo2_base::gates::{GateChip, GateInstructions, RangeInstructions};
use halo2_base::utils::BigPrimeField;
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
use common::{assert_pegs_in_range, hash_pegs};

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
    let range_chip = builder.range_chip();
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

    // Constrain that the pegs and guesses are within range
    assert_pegs_in_range::<F>(&range_chip, ctx, pegs);
    assert_pegs_in_range::<F>(&range_chip, ctx, guesses);

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
    let min_val = |ctx: &mut Context<F>, a: AssignedValue<F>, b: AssignedValue<F>| -> AssignedValue<F> {
        let a_less_than_b = range_chip.is_less_than(ctx, a, b, 4);
        GateChip::<F>::default().select(ctx, a, b, a_less_than_b)
    };

    let count_color = |ctx: &mut Context<F>, pegs: [AssignedValue<F>; 4], color: AssignedValue<F>| -> AssignedValue<F> {
        let eq_vec: Vec<AssignedValue<F>> = pegs.iter()
            .map(|v| GateChip::<F>::default().is_equal(ctx, *v, color))
            .collect();
        GateChip::<F>::default().sum(ctx, eq_vec)
    };

    let min_vals: Vec<AssignedValue<F>> = (0u64..6u64)
        .map(|c| {
            let color = ctx.load_constant(c.into());
            let guess_color = count_color(ctx, guesses, color);
            let code_color = count_color(ctx, pegs, color);
            min_val(ctx, guess_color, code_color)
        })
        .collect();

    let min_sum = GateChip::<F>::default().sum(ctx, min_vals);
    let partial_guesses = GateInstructions::sub(
        &GateChip::<F>::default(),
        ctx,
        min_sum,
        correct_guesses);

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
