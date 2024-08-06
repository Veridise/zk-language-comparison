use dep::std::hash::poseidon2;

fn generate_code_hash(nonce: Field, pegs: [Field; 4]) -> Field {
    let a = [nonce, pegs[0], pegs[1], pegs[2], pegs[3]];
    poseidon2::Poseidon2::hash(a, 5)
}

pub fn set_code(nonce: Field, pegs: [Field; 4]) -> Field {
    // Check bounds
    for i in 0..4 {
        let c = pegs[i];
        assert(c.lt(6), f"Invalid peg color {c}"); // valid = [0,5]
    }
    // Generate the hash
    generate_code_hash(nonce, pegs)
}

// Return value is (# fully correct, # partially correct)
pub fn make_guess(
    nonce: Field,
    pegs: [Field; 4],
    hash: pub Field,
    guess: pub [Field; 4]
) -> (Field, Field) {
    // Validate the hash
    assert(hash == generate_code_hash(nonce, pegs));
    // Tally fully and partially correct count
    let mut correct = 0;
    let mut partial = 0;
    for i in 0..4 {
        let g = guess[i];
        if (g == pegs[i]) {
            correct += 1;
        } else {
            // Check if 'g' matches some pegs[j] s.t. j!=i
            let mut found = false;
            for j in 0..4 {
                if (j != i) {
                    found |= (g==pegs[j]);
                }
            }
            if (found) {
                partial += 1;
            }
        }
    }
    (correct, partial)
}

// fn main() {
// }

#[test(should_fail)]
fn test_set_code_invalid_small() {
    // code pegs should be in the range [0,5]
    let code = [0, 0, 0, -1];
    let _ = set_code(987654321, code);
}

#[test(should_fail)]
fn test_set_code_invalid_large() {
    // code pegs should be in the range [0,5]
    let code = [0, 6, 0, 0];
    let _ = set_code(987654321, code);
}

fn make_guess_with_checks(
    nonce: Field,
    pegs: [Field; 4],
    hash: Field,
    guess: [Field; 4]
) -> (Field, Field) {
    let (correct, partial) = make_guess(nonce, pegs, hash, guess);
    assert((correct + partial).lt(5)); // post-condition of make_guess()
    (correct, partial)
}

#[test]
fn test_good() {
    let codemaker_nonce = 987654321;
    let codemaker_code = [0, 2, 1, 4]; // range = [0,5]

    // At the beginning, codemaker sets the code
    let code_hash = set_code(codemaker_nonce, codemaker_code);
    // Then codebreaker proceeds to make guesses
    let guess = [1, 2, 3, 4];
    let (correct, partial) = make_guess_with_checks(codemaker_nonce, codemaker_code, code_hash, guess);
    assert(correct == 2, f"Expected 2, but got {correct}");
    assert(partial == 1, f"Expected 1, but got {partial}");
    let guess = [0, 2, 1, 4];
    let (correct, partial) = make_guess_with_checks(codemaker_nonce, codemaker_code, code_hash, guess);
    assert(correct == 4, f"Expected 4, but got {correct}");
    assert(partial == 0, f"Expected 0, but got {partial}");
    // If correct == 4 at any point, then the codebreaker wins. If the number of rounds exceeds 10(?) then the codemaker wins.
}
