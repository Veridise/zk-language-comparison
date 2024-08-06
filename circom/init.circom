pragma circom 2.1.8;

include "common.circom";
include "../node_modules/circomlib/circuits/comparators.circom";

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
    /*
    lt[i] = LessThan(3); // This doesn't enforce that pegs[i] has at most 3 bits,
											   // it will just truncate. So, we should enforce this.
    lt[i].in[0] <== pegs[i];
    lt[i].in[1] <== 7;
    lt[i].out === 1;
    
    zeros[i] = IsZero();
    zeros[i].in <== pegs[i];
    zeros[i].out === 0;
    */
    boundsCheck[i] = Num2Bits_strict();
    boundsCheck[i].in <== pegs[i];
    lt[i] = CompConstant(5);
    lt[i].in <== boundsCheck[i].out;
    lt[i].out === 0;
  }
  out <== hasher.out;
}

component main = SetCode();
