# Mastermind (Circom)

## Code Structure

- `common.circom`: contains some generic utility templates used in computing feedback.
- `mastermind.circom`: contains Mastermind specific circuits and main component declaration.
- `examples/*.json`: contains example circuit inputs for witness generation.

## Running the Examples

### Prerequisites

- Requires circom. Follow [these instructions from iden3](https://docs.circom.io/getting-started/installation/)
on how to install circom.
  - Circom requires rust and cargo, which is explained in the above docs.
- Requires circomlib.
- Requires node.

### Command Line

```shell
# Follow above instructions for installing circom and rust
## Install snarkjs
npm install -g snarkjs
## Install circomlib
npm install circomlib

# Compile the circuit
circom mastermind.circom --r1cs --wasm --sym
node ./mastermind_js/generate_witness.js ./mastermind_js/mastermind.wasm examples/input1.json witness.wtns

## To view the computed witness:
snarkjs wtns export json witness.wtns witness.json
## Open mastermind.sym and use the indices to view the computed value in the witness.json
## In this example, the output signals hash, correct_pegs, and partial_pegs are indices 1, 2, and 3 respectively,
## and for examples/input.json, the outputs are "14145853328434050885524917836371294524222664259619069142924766963653570759368", "1", and "2", respectively

# Proving the circuit (https://docs.circom.io/getting-started/proving-circuits/)

snarkjs powersoftau new bn128 16 pot16_0000.ptau -v
snarkjs powersoftau contribute pot16_0000.ptau pot16_0001.ptau --name="First contribution" -v
snarkjs powersoftau prepare phase2 pot16_0001.ptau pot16_final.ptau -v
snarkjs groth16 setup mastermind.r1cs pot16_final.ptau mastermind_0000.zkey
snarkjs zkey contribute mastermind_0000.zkey mastermind_0001.zkey --name="1st Contributor Name" -v
snarkjs zkey export verificationkey mastermind_0001.zkey verification_key.json
snarkjs groth16 prove mastermind_0001.zkey witness.wtns proof.json public.json
snarkjs groth16 verify verification_key.json public.json proof.json
```