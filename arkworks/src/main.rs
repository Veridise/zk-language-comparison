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

// Example function to compute the Poseidon hash
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

fn main() {
    // Example inputs
    let inputs = vec![
        42u64, 1u64, 2u64, 3u64, 4u64,
    ];
    let hash_u64 = compute_poseidon_hash(&inputs);

    // Define the circuit with some inputs
    let circuit = MastermindCircuit::<6, 4> {
        code: [Some(1), Some(2), Some(3), Some(4)],
        nonce: Some(42),
        hash: Some(hash_u64),
        num_partial_correct: Some(2),
        num_fully_correct: Some(2),
        guess: [Some(1), Some(2), Some(4), Some(3)],
    };

    // Needs to implement CryptoRng
    let rng = &mut ChaCha20Rng::seed_from_u64(42); // Use ChaCha20Rng with a fixed seed

    // Create the parameters
    let params = {
        // let rng = &mut ark_std::test_rng();
        Groth16::<Bn254>::setup(circuit.clone(), rng).unwrap()
    };

    // Generate the proof
    let proof = {
        // let rng = &mut ark_std::test_rng();
        Groth16::<Bn254>::prove(&params.0, circuit, rng).unwrap()
    };

    println!("Proof: {:?}", proof);
}
