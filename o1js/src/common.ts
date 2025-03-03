// NCOLORS = 6, NPEGS = 4
import { Field, Provable, Poseidon } from "o1js";

export {
  checkBounds,
  generateCodeHash,
  countCorrectGuesses,
  countPartialGuesses,
};

function generateCodeHash(code: Field[], salt: Field) {
  return Poseidon.hash([...code, salt]);
}

function checkBounds(pegs: Field[]) {
  for (let i = 0; i < 4; i++) {
    pegs[i].assertLessThan(6, `Invalid peg color at index ${i}`);
  }
}

function minimum(f1: Field, f2: Field) {
  const min = Provable.if(f1.lessThan(f2), f1, f2);
  return min;
}

function countCorrectGuesses(guess: Field[], code: Field[]) {
  let correctCount = Field(0);
  for (let i = 0; i < 4; i++) {
    const isCorrect = guess[i].equals(code[i]);
    correctCount = correctCount.add(Provable.if(isCorrect, Field(1), Field(0)));
  }

  return correctCount;
}

function countColors(pegs: Field[], color: Field) {
  let count = Field(0);
  for (let i = 0; i < 4; i++) {
    const isMatch = color.equals(pegs[i]);
    count = count.add(Provable.if(isMatch, count.add(1), Field(0)));
  }

  return count;
}

function countPartialGuesses(
  guess: Field[],
  code: Field[],
  correctCount: Field
) {
  let sum = Field(0);
  for (let c = 0; c < 6; c++) {
    let codeCount = countColors(code, Field(c));
    let guessCount = countColors(guess, Field(c));
    let minCount = minimum(codeCount, guessCount);
    sum = sum.add(minCount);
  }
  let partialCount = sum.sub(correctCount);

  return partialCount;
}
