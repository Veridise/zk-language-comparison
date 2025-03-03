import { Field, Provable, ZkProgram, Struct } from "o1js";
import {
  checkBounds,
  generateCodeHash,
  countCorrectGuesses,
  countPartialGuesses,
} from "./common.js";

export { MastermindProgram, PublicInputs };

class PublicInputs extends Struct({
  codeHash: Field,
  guess: Provable.Array(Field, 4),
}) {}

class PublicOutputs extends Struct({
  correctCount: Field,
  partialCount: Field,
}) {}

let MastermindProgram = ZkProgram({
  name: "make-guess-circuit",
  publicOutput: PublicOutputs,
  publicInput: PublicInputs,

  methods: {
    makeGuess: {
      privateInputs: [Provable.Array(Field, 4), Field],
      async method(publicInputs: PublicInputs, code: Field[], salt: Field) {
        const { codeHash, guess } = publicInputs;

        // Validate the pegs in the code and the guess
        checkBounds(code);
        checkBounds(guess);

        // Compute the code hash
        const computedCodeHash = generateCodeHash(code, salt);

        // Validate the code hash
        computedCodeHash.assertEquals(codeHash);

        // Tally the count of correct guesses
        const correctCount = countCorrectGuesses(guess, code);

        // Tally the count of partial guesses
        const partialCount = countPartialGuesses(guess, code, correctCount);

        // Assert that the total count doesn't exceed the number of pegs
        correctCount.add(partialCount).assertLessThan(5);

        return {
          publicOutput: new PublicOutputs({ correctCount, partialCount }),
        };
      },
    },
  },
});
