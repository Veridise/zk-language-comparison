import { MastermindProgram, PublicInputs } from "./mastermind.js";
import { assert, Field } from "o1js";
import { generateCodeHash } from "./common.js";

console.time("Compiling MastermindProgram...");
await MastermindProgram.compile();
console.timeEnd("Compiling MastermindProgram...");

const cs = await MastermindProgram.analyzeMethods();
console.log("\nMastermind Program Constraints: ", cs.makeGuess.summary());

// Example 1
let proof;
let code = [0, 2, 1, 4].map(Field);
let salt = Field.random();

let codeHash = generateCodeHash(code, salt);
let guess = [1, 2, 3, 4].map(Field);
let publicInputs = new PublicInputs({ codeHash, guess });

({ proof } = await MastermindProgram.makeGuess(publicInputs, code, salt));

let isValid = await MastermindProgram.verify(proof);

assert(isValid, "Proof is not valid!");
let { correctCount, partialCount } = proof.publicOutput;
assert(correctCount.toBigInt() == 2n, `expected 2 but got ${correctCount}`);
assert(partialCount.toBigInt() == 1n, `expected 1 but got ${partialCount}`);

// Example 2
code = [0, 2, 1, 4].map(Field);
salt = Field.random();

codeHash = generateCodeHash(code, salt);
guess = [0, 2, 1, 4].map(Field);
publicInputs = new PublicInputs({ codeHash, guess });

({ proof } = await MastermindProgram.makeGuess(publicInputs, code, salt));

isValid = await MastermindProgram.verify(proof);

assert(isValid, "Proof is not valid!");
correctCount = proof.publicOutput.correctCount;
partialCount = proof.publicOutput.partialCount;
assert(correctCount.toBigInt() == 4n, `expected 4 but got ${correctCount}`);
assert(partialCount.toBigInt() == 0n, `expected 0 but got ${partialCount}`);
