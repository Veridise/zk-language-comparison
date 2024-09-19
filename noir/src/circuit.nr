use dep::std::hash::poseidon2;

// NCOLORS = 6, NPEGS = 4

pub fn generate_code_hash(nonce: Field, pegs: [Field; 4]) -> Field {
    let a = [nonce, pegs[0], pegs[1], pegs[2], pegs[3]];
    poseidon2::Poseidon2::hash(a, 5)
}

pub fn check_bounds(pegs: [Field; 4]) {
    for i in 0..4 {
        let c = pegs[i];
        assert(c.lt(6), f"Invalid peg color {c}"); // valid = [0,5]
    }
}

pub fn set_code(nonce: Field, pegs: [Field; 4]) -> Field {
    // Check bounds
    check_bounds(pegs);
    // Generate the hash
    generate_code_hash(nonce, pegs)
}

fn minimum(a: Field, b: Field) -> Field {
    if (a.lt(b)) {
        a
    } else {
        b
    }
}

fn count_colors(pegs: [Field; 4], color: Field) -> Field {
    let mut count = 0;
    for i in 0..pegs.len() {
        if (color == pegs[i]) {
            count += 1;
        }
    }
    count
}

// Return value is (# fully correct, # partially correct)
pub fn make_guess(
    nonce: Field,
    pegs: [Field; 4],
    hash: Field, // public
    guess: [Field; 4] // public
) -> (Field, Field) {
    // Validate the bounds of the guess
    check_bounds(guess);
    // Validate the hash (and bounds of pegs)
    assert(hash == generate_code_hash(nonce, pegs));
    // Tally fully correct count
    let mut correct = 0;
    for i in 0..4 {
        if (guess[i] == pegs[i]) {
            correct += 1;
        }
    }
    // Tally partially correct count
    let mut sum = 0;
    for c in 0..6 {
        let color = c as Field;
        let guess_count = count_colors(guess, color);
        let pegs_count = count_colors(pegs, color);
        let min_count = minimum(guess_count, pegs_count);
        sum += min_count;
    }
    let partial = sum - correct;
    (correct, partial)
}

pub fn make_guess_with_checks(
    nonce: Field,
    pegs: [Field; 4],
    hash: Field,
    guess: [Field; 4]
) -> (Field, Field) {
    let (correct, partial) = make_guess(nonce, pegs, hash, guess);
    assert((correct + partial).lt(5)); // post-condition of make_guess()
    (correct, partial)
}
