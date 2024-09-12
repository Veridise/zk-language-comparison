use ark_bn254::Bn254;
use ark_crypto_primitives::sponge::CryptographicSponge;
use ark_snark::SNARK;
use ark_snark::CircuitSpecificSetupSNARK;
use ark_groth16::Groth16;
use arkworks::get_poseidon_config;
use arkworks::MastermindCircuit;
use rand_chacha::ChaCha20Rng;
use ark_std::rand::SeedableRng;

use ark_bn254::Fr;
use ark_crypto_primitives::sponge::poseidon::PoseidonSponge;
// Define the parameters for Poseidon hash (for Bn254)
type PoseidonSpongeBn254 = PoseidonSponge<Fr>;

// Example function to compute the Poseidon hash so we can compute the input hash.
fn compute_poseidon_hash(inputs: &[u64]) -> u64 {
    // Create Poseidon parameters (configuration specific to Bn254)
    let params = get_poseidon_config();

    let mut sponge = PoseidonSpongeBn254::new(&params);
    for i in inputs {
        sponge.absorb(i);
    }

    // Finalize the hash and get the output
    let hash = sponge.squeeze_bytes(8);

    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&hash);

    // Convert to u64
    u64::from_le_bytes(bytes)
}

fn fill_in_hash(circuit: &mut MastermindCircuit::<6, 4>) {
    let inputs = vec![
        circuit.nonce.unwrap(),
        circuit.code[0].unwrap(), circuit.code[1].unwrap(), circuit.code[2].unwrap(), circuit.code[3].unwrap(),
    ];
    let hash_u64 = compute_poseidon_hash(&inputs);
    circuit.hash = Some(hash_u64);
}

fn validate(rng: &mut ChaCha20Rng, circuit: &MastermindCircuit::<6, 4>) {
    // Create the parameters.
    let params = Groth16::<Bn254>::setup(circuit.clone(), rng).unwrap();

    // Generate the proof. This will fail if the constraints are violated.
    let proof = Groth16::<Bn254>::prove(&params.0, circuit.clone(), rng).unwrap();

    println!("Proof: {:?}", proof);
}

fn main() {
    // Example inputs
    let mut test1 = MastermindCircuit::<6, 4> {
        code: [Some(0), Some(0), Some(0), Some(0)],
        nonce: Some(42),
        hash: Some(0),
        num_partial_correct: Some(0),
        num_fully_correct: Some(0),
        guess: [Some(1), Some(2), Some(4), Some(3)],
    };
    fill_in_hash(&mut test1);

    let mut test2 = MastermindCircuit::<6, 4> {
        code: [Some(1), Some(2), Some(3), Some(4)],
        nonce: Some(43),
        hash: Some(0),
        num_partial_correct: Some(2),
        num_fully_correct: Some(2),
        guess: [Some(1), Some(2), Some(4), Some(3)],
    };
    fill_in_hash(&mut test2);

    // The RNG needs to implement CryptoRng
    // - Use ChaCha20Rng with a fixed seed
    let rng = &mut ChaCha20Rng::seed_from_u64(42);

    validate(rng, &test1);
    validate(rng, &test2);
}
