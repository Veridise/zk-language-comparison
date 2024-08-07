use clap::Parser;
use halo2_base::gates::circuit::builder::BaseCircuitBuilder;
use halo2_base::gates::{GateChip, GateInstructions, RangeInstructions};
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

const MAX_COLOR: usize = 5;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitInput {
    nonce: String, // field element, but easier to deserialize as a string
    pegs: [String; 4],
}

fn codebreaker_init<F: BigPrimeField>(
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

    for i in 0..4 {
        range_chip.range_check(ctx, pegs[i], 3); //TODO: This means there are 8 colors
    }

    let hash = hash_pegs(ctx, nonce, pegs);
    make_public.push(hash);

    println!("Hash output: {:?}", hash.value());
}

fn main() {
    env_logger::init();

    let args = Cli::parse();

    // run different zk commands based on the command line arguments
    run(codebreaker_init, args);
}
