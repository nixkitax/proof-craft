use ark_bls12_381::Fr as ScalarField;
use ark_ff::UniformRand;
use merkletree::merkle_tree::MerkleTree;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = ark_std::test_rng();

    // Create a vector of random scalar field elements.
    let values = vec![
        ScalarField::rand(&mut rng),
        ScalarField::rand(&mut rng),
        ScalarField::rand(&mut rng),
        ScalarField::rand(&mut rng),
    ];

    // Step 1: Construct the Merkle Tree.
    let tree = MerkleTree::new(values.clone());
    println!("Merkle Tree constructed successfully!");

    // Step 2: Retrieve the root hash of the Merkle Tree.
    let root = tree.root();
    println!("Merkle Tree root: {:?}", root);

    // Step 3: Generate a Merkle proof for a specific leaf index.
    let index = 1; // Verify the second value (index 1).
    let proof = tree.generate_proof(index);
    println!("Proof generated for index {}: {:?}", index, proof);

    // Step 4: Verify the Merkle proof.
    let is_valid = MerkleTree::verify_proof(root, index, &values[index], &proof);
    println!("Is the proof valid? {}", is_valid);

    // Step 5: Test with an invalid value.
    let invalid_value = ScalarField::rand(&mut rng); // A random value not in the tree.
    let is_invalid = MerkleTree::verify_proof(root, index, &invalid_value, &proof);
    println!("Is the proof valid for an invalid value? {}", is_invalid);

    Ok(())
}