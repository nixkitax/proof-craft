use ark_serialize::CanonicalSerialize;
use ark_bls12_381::Fr as ScalarField;
use crate::hash::hash;

// A Merkle Tree is a cryptographic data structure that allows efficient and secure verification
// of the contents of a dataset. It is constructed by recursively hashing pairs of nodes until
// a single root hash is obtained. This root hash serves as a commitment to the entire dataset.
pub struct MerkleTree {
    leaves: Vec<Vec<u8>>, // The leaves of the tree, represented as hashes of the original values.
    root: Vec<u8>,        // The root hash of the Merkle Tree, serving as the commitment.
}

impl MerkleTree {
    // Constructs a Merkle Tree from a vector of scalar field elements.
    // Each element is serialized and hashed to form the leaves of the tree.
    // The tree is then built by recursively hashing pairs of nodes until a single root is obtained.
    pub fn new(values: Vec<ScalarField>) -> Self {
        // Step 1: Serialize each value into bytes and compute its hash to form the leaves.
        let leaves: Vec<Vec<u8>> = values.iter()
            .map(|v| {
                let mut bytes = Vec::new();
                v.serialize_uncompressed(&mut bytes).unwrap(); // Serialize the value into bytes.
                bytes // The serialized bytes represent the leaf node.
            })
            .collect();

        // Step 2: Build the tree levels by recursively hashing pairs of nodes.
        let mut current_level = leaves.clone();
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            // Combine nodes in pairs to compute their parent hash.
            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i + 1] // Use the right sibling if it exists.
                } else {
                    &current_level[i] // Duplicate the last node if the number of nodes is odd.
                };
                
                next_level.push(hash(left, right)); // Compute the hash of the parent node.
            }
            
            current_level = next_level; // Move to the next level of the tree.
        }
        
        // The root is the single remaining node at the top of the tree.
        let root = current_level[0].clone();
        Self { leaves, root }
    }

    // Returns the root hash of the Merkle Tree.
    // The root hash serves as a cryptographic commitment to the entire dataset.
    pub fn root(&self) -> &[u8] {
        &self.root
    }

    // Generates a Merkle proof for a specific leaf index.
    // A Merkle proof consists of the sibling hashes required to reconstruct the root hash
    // from the target leaf. This proof allows verification of the leaf's inclusion in the tree.
    pub fn generate_proof(&self, index: usize) -> Vec<Vec<u8>> {
        let mut proof = Vec::new();
        let mut current_index = index;
        let mut current_level = self.leaves.clone();

        while current_level.len() > 1 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1 // The sibling is the right node if the current index is even.
            } else {
                current_index - 1 // The sibling is the left node if the current index is odd.
            };

            // Add the sibling hash to the proof.
            if sibling_index < current_level.len() {
                proof.push(current_level[sibling_index].clone());
            }

            // Move to the parent level.
            current_index /= 2;
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i + 1]
                } else {
                    &current_level[i]
                };
                next_level.push(hash(left, right)); // Compute the parent hash.
            }
            current_level = next_level;
        }

        proof
    }

    // Verifies a Merkle proof for a specific leaf index.
    // The proof is valid if the computed root hash matches the actual root hash of the tree.
    pub fn verify_proof(root: &[u8], index: usize, value: &ScalarField, proof: &[Vec<u8>]) -> bool {
        // Step 1: Serialize the value and compute its hash.
        let mut current_hash = {
            let mut bytes = Vec::new();
            value.serialize_uncompressed(&mut bytes).unwrap();
            bytes
        };

        // Step 2: Reconstruct the root hash using the proof.
        let mut current_index = index;
        for sibling in proof {
            if current_index % 2 == 0 {
                // If the index is even, the sibling is the right node.
                current_hash = hash(&current_hash, sibling);
            } else {
                // If the index is odd, the sibling is the left node.
                current_hash = hash(sibling, &current_hash);
            }
            current_index /= 2; // Move to the parent level.
        }

        // Step 3: Compare the computed root hash with the actual root hash.
        current_hash == root
    }
}
