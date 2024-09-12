# ZK Mastermind: Noir

A Halo2 implementation of the initialization and validation of a mastermind game. This uses the halo2-scaffolding to remove the need for some of the boilerplate.

## Code Structure

The relevant code is in the `examples/` directory.
This is so that the commands can be easily ran with the `--example` flag of `cargo run`.

## Running the Examples

### Prerequisites

Install cargo and rust.

### Command Line Instructions

To run the init:
```shell
LOOKUP_BITS=3 cargo run --example init -- --name init -k 6 mock
```

To run the validation:
```shell
LOOKUP_BITS=3 cargo run --example validate -- --name validate -k 6 mock
```