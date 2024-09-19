use clap::Parser;
use halo2_base::{gates::circuit::builder::BaseCircuitBuilder, utils::BigPrimeField};
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

    // Ensure pegs are in the range.
    assert_pegs_in_range::<F>(&range_chip, ctx, pegs);

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
