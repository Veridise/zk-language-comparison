# Halo2

A Halo2 implementation of the initialization and validation of a mastermind game. This uses the halo2-scaffolding to remove the need for some of the boilerplate.

The relevant code is in the `examples/` directory. This is so that the commands can be easily ran with the --example flag of cargo run.


To run the init:
```
LOOKUP_BITS=3 cargo run --example init -- --name init -k 6 mock
```


To run the validation:
```
LOOKUP_BITS=3 cargo run --example validate -- --name validate -k 6 mock
```