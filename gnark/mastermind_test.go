package mastermind

import (
	"encoding/binary"
	"fmt"
	"math/big"
	"math/rand"
	"reflect"
	"testing"

	"github.com/consensys/gnark-crypto/ecc"
	"github.com/consensys/gnark-crypto/ecc/bn254/fr/mimc"
	"github.com/consensys/gnark/backend/groth16"
	"github.com/consensys/gnark/frontend"
	"github.com/consensys/gnark/frontend/cs/r1cs"
)

type UintCode = [CODE_SIZE]uint64

func ToSignalCode(u UintCode) (c Code) {
	for i := 0; i < len(u); i++ {
		c[i] = u[i]
	}
	return
}

// Hash paramaters for the hash assignment.
func AssignmentHash(vars ...uint64) (h *big.Int, err error) {
	hash := mimc.NewMiMC()

	bytes := make([]byte, 8)
	for _, v := range vars {
		binary.BigEndian.PutUint64(bytes, v)
		if _, err := hash.Write(bytes); err != nil {
			return nil, err
		}
	}

	h = big.NewInt(0)
	hBytes := hash.Sum(h.Bytes())
	h.SetBytes(hBytes)
	return h, nil
}

func CreateCodeAssignment(t *testing.T, nonce uint64, code UintCode) CodemakerCode {
	vars := []uint64{nonce}
	for _, peg := range code {
		vars = append(vars, peg)
	}
	hash, err := AssignmentHash(vars...)
	if err != nil {
		t.Error(err)
	}
	t.Logf("hash is %v\n", hash)
	return CodemakerCode{
		Nonce: nonce,
		Pegs:  ToSignalCode(code),
		Hash:  hash,
	}
}

func RandomPeg() uint64 {
	return uint64(rand.Intn(NUM_PEG_CHOICES))
}

func RandomNonce() uint64 {
	return uint64(rand.Uint32())
}

func RandomCode() UintCode {
	return UintCode{
		RandomPeg(), RandomPeg(), RandomPeg(), RandomPeg(),
	}
}

func ComputeAnswer(answer, guess UintCode) (numCorrect, numPartial uint64) {
	for i := 0; i < len(answer); i++ {
		if answer[i] == guess[i] {
			numCorrect++
		}
	}

	count := func(color uint64, code UintCode) uint64 {
		var c uint64 = 0
		for i := 0; i < len(code); i++ {
			if code[i] == color {
				c++
			}
		}
		return c
	}

	var minSum uint64 = 0
	var c uint64
	for c = 0; c < NUM_PEG_CHOICES; c++ {
		minSum += min(count(c, answer), count(c, guess))
	}
	numPartial = minSum - numCorrect
	return
}

func GenerateTestCase(t *testing.T) MastermindCircuit {
	nonce := RandomNonce()
	code := RandomCode()
	guess := RandomCode()
	numCorrect, numPartial := ComputeAnswer(code, guess)
	t.Logf("%#v %#v %#v => %#v %#v\n", nonce, code, guess, numCorrect, numPartial)

	return MastermindCircuit{
		Code: CreateCodeAssignment(t, nonce, code),
		Guess: CodebreakerGuess{
			Pegs: ToSignalCode(guess),
		},
		Response: CodemakerResponse{
			NumCorrect: numCorrect,
			NumPartial: numPartial,
		},
	}
}

func TestExampleMastermindCircuit(t *testing.T) {
	// Setup the witness
	assignment := MastermindCircuit{
		Code: CreateCodeAssignment(t, 54321, UintCode{0, 0, 0, 0}),
		Guess: CodebreakerGuess{
			Pegs: Code{0, 0, 0, 0},
		},
		Response: CodemakerResponse{
			NumCorrect: 4,
			NumPartial: 0,
		},
	}

	err := PerformCircuitTest(&assignment)
	if err != nil {
		t.Error(err)
	}
}

func TestRandomCircuits(t *testing.T) {
	for i := 0; i < 10; i++ {
		assignment := GenerateTestCase(t)

		err := PerformCircuitTest(&assignment)
		if err != nil {
			t.Error(err)
		}
	}
}

func PerformCircuitTest[C frontend.Circuit](assignedCircuit C) error {
	// Create a new instance of the type C using reflection, since we need
	// an unassigned circuit to compile.
	circuitValue := reflect.New(reflect.TypeOf(assignedCircuit).Elem())
	if circuitValue.IsNil() {
		return fmt.Errorf("unable to create instance of type %T", assignedCircuit)
	}
	circuit := circuitValue.Interface().(C)
	r1cs, err := frontend.Compile(ecc.BN254.ScalarField(), r1cs.NewBuilder, circuit)
	if err != nil {
		return err
	}
	// generating pk, vk
	pk, vk, err := groth16.Setup(r1cs)
	if err != nil {
		return err
	}

	// witness
	witness, err := frontend.NewWitness(assignedCircuit, ecc.BN254.ScalarField())
	if err != nil {
		return err
	}
	publicWitness, err := witness.Public()
	if err != nil {
		return err
	}
	// generate the proof
	proof, err := groth16.Prove(r1cs, pk, witness)
	if err != nil {
		return err
	}

	// verify the proof
	err = groth16.Verify(proof, vk, publicWitness)
	if err != nil {
		return err
	}

	return nil
}
