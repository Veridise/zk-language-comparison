pragma circom 2.1.8;

include "../node_modules/circomlib/circuits/poseidon.circom";

template GenerateCodeHash() {
  signal input nonce;
  signal input pegs[4];
  signal output out;

  component poseidon = Poseidon(5);
  poseidon.inputs[0] <== nonce;

  for (var i = 0; i < 4; i++) poseidon.inputs[i+1] <== pegs[i];
  out <== poseidon.out;
}
