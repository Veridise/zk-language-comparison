pragma circom 2.1.8;

include "common.circom";
include "./node_modules/circomlib/circuits/comparators.circom";
include "./node_modules/circomlib/circuits/gates.circom";
include "./node_modules/circomlib/circuits/Mux1.circom";

/*
  Validate that the code is legal.
*/
template ValidateCode(NPEGS, NCOLORS) {
  signal input pegs[NPEGS];

  component boundsCheck[NPEGS];
  signal isGt[NPEGS];
  for (var i = 0; i < NPEGS; i++) {
    // Check that peg is in [0, NCOLORS-1]
    boundsCheck[i] = Num2Bits_strict();
    boundsCheck[i].in <== pegs[i];

    // Returns 1 if pegs[i] > NCOLORS - 1, so out === 0 enforces that the peg
    // is <= NCOLORS - 1. The 0 <= peg[i] bound is implicit.
    isGt[i] <== CompConstant(NCOLORS - 1)(boundsCheck[i].out);
    isGt[i] === 0;
  }
}

/*
  Generate the hash of the nonce + pegs, preventing dictionary attacks.
  Does not check that the code is validate; check must be done elsewhere.
*/
template GenerateCodeHash(NPEGS) {
  signal input nonce;
  signal input pegs[NPEGS];
  signal output out;

  component poseidon = Poseidon(NPEGS + 1);
  poseidon.inputs[0] <== nonce;

  for (var i = 0; i < NPEGS; i++) {
    poseidon.inputs[i+1] <== pegs[i];
  }
  out <== poseidon.out;
}

/*
  Compute the number of correct guesses (black pegs).
*/
template CorrectGuesses(NPEGS) {
  signal input code[NPEGS];
  signal input guess[NPEGS];
  signal output correct;

  signal isEq[NPEGS];
  var sum = 0;
  for (var i = 0; i < NPEGS; i++) {
    isEq[i] <== IsEqual()([code[i], guess[i]]); // Using anonymous component syntax for brevity
    sum += isEq[i]; // However, you cannot directly use anonymous component outputs with variables
  }

  correct <== sum;
}

/*
  Compute the number of partial guesses (white pegs) based on the minimum formula:
    - For each color, compute the minimum of the number of that color peg in the
    codebreaker guess and the codemaker code.
    - Compute the sum of these minimums across all colors, then subtract the
    number of correct guesses (black pegs) computed earlier.
*/
template PartialGuesses(NPEGS, NCOLORS) {
  signal input code[NPEGS];
  signal input guess[NPEGS];
  signal input correct;
  signal output partial;

  signal min[NCOLORS];
  var color_match_sum = 0;
  for (var color = 0; color < NCOLORS; color++) {
    var codeColor = CountConst(NPEGS, color)(code);
    var guessColor = CountConst(NPEGS, color)(guess);

    min[color] <== Minimum()(codeColor, guessColor);
    color_match_sum += min[color];
  }

  partial <== color_match_sum - correct;
}

/*
  Given the nonce, codemaker code, and codebreaker guess, compute the hash
  of the game and the feedback.
*/
template Mastermind(NPEGS, NCOLORS) {
  // private
  signal input nonce;
  signal input code[4];

  // public
  signal input guess[4];

  signal output hash;
  signal output correct_pegs; // The number of correct codes
  signal output partial_pegs; // The number of correct colors in wrong positions

  // Validate the pegs in the code and the guess
  ValidateCode(NPEGS, NCOLORS)(code);
  ValidateCode(NPEGS, NCOLORS)(guess);

  // Compute the hash
  hash <== GenerateCodeHash(NPEGS)(nonce, code);

  // Tally the correct guesses
  correct_pegs <== CorrectGuesses(NPEGS)(code, guess);

  // Tallying the partial count
  partial_pegs <== PartialGuesses(NPEGS, NCOLORS)(code, guess, correct_pegs);
}

component main{public [guess]} = Mastermind(4, 6);
