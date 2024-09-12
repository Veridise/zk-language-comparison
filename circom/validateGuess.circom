pragma circom 2.1.8;

include "common.circom";
include "../node_modules/circomlib/circuits/comparators.circom";
include "../node_modules/circomlib/circuits/gates.circom";
include "../node_modules/circomlib/circuits/Mux1.circom";

template ValidateCode() {
    signal input nonce;
    signal input pegs[4];

    // public
    signal input hash;
    signal input guess[4];

    signal output correct_pegs; // The number of correct codes
    signal output partial_pegs; // The number of correct colors in wrong positions

    // Validate the hash
    component hasher = GenerateCodeHash();
    hasher.nonce <== nonce;
    hasher.pegs <== pegs;
    hasher.out === hash;

    // Tally the correct guesses
    var total;
    component eq[4];
    for (var i = 0; i < 4; i++) {
        eq[i] = IsEqual();
        eq[i].in[0] <== pegs[i];
        eq[i].in[1] <== guess[i];

        total += eq[i].out;
    }
    correct_pegs <== total;

    // Tallying the partial count
    component allEq[4][4];
    component eqMux[4][4];
    component z[4];
    component notZ[4];
    component sel[4][4];

    var partialTotal;
    for (var i = 0; i < 4; i++) {
	    var totalPartial = 0;
	    // For each position i, get the eq value for all other positions j != i
	    for (var j = 0; j < 4; j++) {
		    allEq[i][j] = IsEqual();
		    allEq[i][j].in[0] <== pegs[i];
		    allEq[i][j].in[1] <== guess[j];

		    eqMux[i][j] = Mux1();
		    eqMux[i][j].c[0] <== allEq[i][j].out;
		    eqMux[i][j].c[1] <== 0;

		    sel[i][j] = AND();
		    sel[i][j].a <== (j != i); // fine b/c of compile time unrolling
		    sel[i][j].b <== eq[i].out;

		    eqMux[i][j].s <== sel[i][j].out;

			totalPartial += eqMux[i][j].out;
	    }

	    // If the totalPartial >= 1, then we emit one "partially correct" guess
	    z[i] = IsZero();
	    z[i].in <== totalPartial;
	    //notZ[i] = NOT();
	    //notZ[i].in <== z[i].out;
	    partialTotal += (1 - z[i].out);
    }
     partial_pegs <== partialTotal;
}

component main{public [hash, guess]} = ValidateCode();
