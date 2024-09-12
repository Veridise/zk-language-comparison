pragma circom 2.1.8;

include "common.circom";
include "./node_modules/circomlib/circuits/comparators.circom";

template SetCode() {
  signal input nonce; // salt the hash to prevent dictionary attack
  signal input pegs[4]; // code pegs, each is a value in [1,6], repetition allowed

  signal output out;

  component boundsCheck[4];
  component lt[4];
  // component zeros[4];
  component hasher = GenerateCodeHash();
  hasher.nonce <== nonce;
  hasher.pegs <== pegs;
  for (var i = 0; i < 4; i++) {
    // Check that peg is in [0,5]
    boundsCheck[i] = Num2Bits_strict();
    boundsCheck[i].in <== pegs[i];
    lt[i] = CompConstant(5);
    lt[i].in <== boundsCheck[i].out;
    lt[i].out === 0;
  }
  out <== hasher.out;
}

component main = SetCode();
