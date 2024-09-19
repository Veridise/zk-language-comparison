package mastermind

import (
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/std/hash/mimc"
)

const (
	CODE_SIZE       = 4 // The number of "digits"/"terms"/"pegs" in the code
	NUM_PEG_CHOICES = 6 // The number of different types of pegs. Usually 6 colored pegs.
)

// Asserts that the given peg is a legal choice.
func AssertPegIsLegal(api frontend.API, peg frontend.Variable) {
	// Assert that peg is in range [0, NUM_PEG_CHOICES - 1],
	// equivalent to [0, NUM_PEG_CHOICES).
	api.AssertIsLessOrEqual(peg, NUM_PEG_CHOICES-1)
}

// Assert that all the given pegs are legal.
func AssertPegsAreLegal(api frontend.API, pegs ...frontend.Variable) {
	for _, peg := range pegs {
		AssertPegIsLegal(api, peg)
	}
}

type Code = [CODE_SIZE]frontend.Variable

// The player's, i.e. codebreaker's, input. The player simply provides their
// guess of what the code is.
type CodebreakerGuess struct {
	Pegs Code `gnark:",public"`
}

// Assert that the guess is legal.
func (c *CodebreakerGuess) AssertIsValid(api frontend.API) {
	AssertPegsAreLegal(api, c.Pegs[:]...)
}

type CodemakerCode struct {
	// A nonce added to the code hash.
	// Used to prevent against dictionary attacks on the code hash, as the number
	// of possible codes is relatively small and could be otherwise brute-forced.
	Nonce frontend.Variable `gnark:",secret"`
	Pegs  Code              `gnark:",secret"`
	Hash  frontend.Variable `gnark:",public"`
}

/*
Compute the hash of the secret code values (the game-specific nonce and the selected code
for the game).
*/
func (c *CodemakerCode) ComputeCodeHash(api frontend.API) (hash frontend.Variable, err error) {
	hashFn, err := mimc.NewMiMC(api)
	if err != nil {
		return nil, err
	}
	hashFn.Write(c.Nonce)
	hashFn.Write(c.Pegs[:]...)
	return hashFn.Sum(), nil
}

/*
Checks that the Codemaker has given us a fair code. This means it needs to match
the public hash (so the code doesn't change in the middle of the game) and that
the code is a legal code value.
*/
func (c *CodemakerCode) AssertIsValid(api frontend.API) error {
	if computedHash, err := c.ComputeCodeHash(api); err != nil {
		return err
	} else {
		// Check nonce+code is as expected.
		api.AssertIsEqual(computedHash, c.Hash)
		// var _ = computedHash
		// Check that all pegs are legal.
		for _, peg := range c.Pegs {
			AssertPegIsLegal(api, peg)
		}
	}
	return nil
}

/*
The response from the codemaker on the validity of the guess. There are two
parts of the guess:
- "NumCorrect" is the number of pegs that are the correct choice ("color") and
in the correct position.
- "NumPartial" is the number of pegs that are the correct value but in the wrong
position.
*/
type CodemakerResponse struct {
	NumCorrect frontend.Variable `gnark:",public"`
	NumPartial frontend.Variable `gnark:",public"`
}

/*
Compute the number of correct pegs (black pegs).
*/
func ComputeCorrectPegs(api frontend.API, code *CodemakerCode, guess *CodebreakerGuess) frontend.Variable {
	var numCorrect frontend.Variable = 0
	for i := 0; i < CODE_SIZE; i++ {
		isEq := api.IsZero(api.Cmp(code.Pegs[i], guess.Pegs[i]))
		numCorrect = api.Add(numCorrect, isEq)
	}
	return numCorrect
}

func Min(api frontend.API, a, b frontend.Variable) frontend.Variable {
	cmp := api.Cmp(a, b)
	aLessThanB := api.IsZero(api.Add(cmp, 1))
	return api.Select(aLessThanB, a, b)
}

/*
Count the number of times the color appears in the code.
*/
func CountColor(api frontend.API, code *Code, color frontend.Variable) frontend.Variable {
	var count frontend.Variable = 0
	for i := 0; i < CODE_SIZE; i++ {
		isEq := api.IsZero(api.Cmp(code[i], color))
		count = api.Add(count, isEq)
	}
	return count
}

/*
Compute the number of partial guesses (white pegs) based on the minimum formula:
  - For each color, compute the minimum of the number of that color peg in the
    codebreaker guess and the codemaker code.
  - Compute the sum of these minimums across all colors, then subtract the
    number of correct guesses (black pegs) computed earlier.
*/
func ComputePartialPegs(api frontend.API, code *CodemakerCode, guess *CodebreakerGuess, numCorrect frontend.Variable) frontend.Variable {
	var minSum frontend.Variable = 0
	for i := 0; i < NUM_PEG_CHOICES; i++ {
		codeCount := CountColor(api, &code.Pegs, i)
		guessCount := CountColor(api, &guess.Pegs, i)
		countMin := Min(api, codeCount, guessCount)
		minSum = api.Add(minSum, countMin)
	}
	return api.Sub(minSum, numCorrect)
}

/*
Assert that the codemaker response is a valid response.
A response is valid if the number of outputs (fully correct and partially correct)
*/
func (c *CodemakerResponse) AssertIsValid(api frontend.API, code *CodemakerCode, guess *CodebreakerGuess) {
	// We could do a sanity check that c.NumCorrect + c.NumPartial < number of pegs,
	// but it's not strictly necessary---we'll discover this error via the below
	// computation.

	// Computing the number of correct and partial guesses from the codebreaker guess
	numCorrect := ComputeCorrectPegs(api, code, guess)
	numPartial := ComputePartialPegs(api, code, guess, numCorrect)

	api.AssertIsEqual(c.NumCorrect, numCorrect)
	api.AssertIsEqual(c.NumPartial, numPartial)
}

type MastermindCircuit struct {
	// Secrets
	Code CodemakerCode
	// Public
	Guess    CodebreakerGuess
	Response CodemakerResponse
}

func (m *MastermindCircuit) Define(api frontend.API) error {
	/*
		First, check that the codemaker code is valid.
		This ensures that we're all on the same page with what game we're playing
		and that we're playing a fair game.
	*/
	m.Code.AssertIsValid(api)
	/*
		Second, check that the codebreaker submitted a valid code.
	*/
	m.Guess.AssertIsValid(api)
	/*
		Now that we've validated the codebreaker and codemaker inputs, we now
		check if the response is valid.
		This is the last thing we need to do. We've now asserted the consistency
		of all fields with one another.
	*/
	m.Response.AssertIsValid(api, &m.Code, &m.Guess)

	return nil
}
