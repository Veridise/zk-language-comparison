mod circuit;

#[test(should_fail)]
fn test_set_code_invalid_small() {
    // code pegs should be in the range [0,5]
    let code = [0, 0, 0, -1];
    let _ = circuit::set_code(987654321, code);
}

#[test(should_fail)]
fn test_set_code_invalid_large() {
    // code pegs should be in the range [0,5]
    let code = [0, 6, 0, 0];
    let _ = circuit::set_code(987654321, code);
}

#[test]
fn test_good() {
    let codemaker_nonce = 987654321;
    let codemaker_code = [0, 2, 1, 4]; // range = [0,5]

    // At the beginning, codemaker sets the code
    let code_hash = circuit::set_code(codemaker_nonce, codemaker_code);
    // Then codebreaker proceeds to make guesses
    let guess = [1, 2, 3, 4];
    let (correct, partial) = circuit::make_guess_with_checks(codemaker_nonce, codemaker_code, code_hash, guess);
    assert(correct == 2, f"Expected 2, but got {correct}");
    assert(partial == 1, f"Expected 1, but got {partial}");
    let guess = [0, 2, 1, 4];
    let (correct, partial) = circuit::make_guess_with_checks(codemaker_nonce, codemaker_code, code_hash, guess);
    assert(correct == 4, f"Expected 4, but got {correct}");
    assert(partial == 0, f"Expected 0, but got {partial}");
    // If correct == 4 at any point, then the codebreaker wins. If the number of rounds exceeds 10(?) then the codemaker wins.
}
