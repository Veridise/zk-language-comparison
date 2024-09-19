pragma circom 2.1.8;

include "./node_modules/circomlib/circuits/bitify.circom";
include "./node_modules/circomlib/circuits/comparators.circom";
include "./node_modules/circomlib/circuits/compconstant.circom";
include "./node_modules/circomlib/circuits/mux1.circom";
include "./node_modules/circomlib/circuits/poseidon.circom";

/*
  Assumes the range of a and b have already been checked.
*/
template Minimum() {
  signal input a, b;

  signal aGtB <== GreaterThan(252)([a, b]);

  signal output min <== Mux1()([a, b], aGtB); // Will select a if aGtB == 0, b otherwse.
}

/*
  Count the number of instances of constant C in the input array of size N.
*/
template CountConst(N, C) {
  signal input inp[N];
  signal output count;

  signal isEq[N];
  var sum = 0;
  for (var i = 0; i < N; i++) {
    isEq[i] <== IsEqual()([C, inp[i]]);
    sum += isEq[i];
  }
  count <== sum;
}
