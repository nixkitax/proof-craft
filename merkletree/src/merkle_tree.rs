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
    pub fn new(values: Vec<ScalarField>) -> Self {
        let leaves: Vec<Vec<u8>> = values.iter()
            .map(|v| {
                let mut bytes = Vec::new();
                v.serialize_uncompressed(&mut bytes).unwrap();
                bytes
            })
            .collect();

        let mut current_level = leaves.clone();
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i + 1]
                } else {
                    &current_level[i]
                };
                next_level.push(hash(left, right));
            }
            current_level = next_level;
        }

        let root = current_level[0].clone();
        Self { leaves, root }
    }

    // Returns the root hash of the Merkle Tree.
    pub fn root(&self) -> &[u8] {
        &self.root
    }

    // Generates a Merkle proof for a specific leaf index.
    pub fn generate_proof(&self, index: usize) -> Vec<Vec<u8>> {
        let mut proof = Vec::new();
        let mut current_index = index;
        let mut current_level = self.leaves.clone();

        while current_level.len() > 1 {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            if sibling_index < current_level.len() {
                proof.push(current_level[sibling_index].clone());
            }

            current_index /= 2;
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                let left = &current_level[i];
                let right = if i + 1 < current_level.len() {
                    &current_level[i + 1]
                } else {
                    &current_level[i]
                };
                next_level.push(hash(left, right));
            }
            current_level = next_level;
        }

        proof
    }

    // Verifies a Merkle proof for a specific leaf index.
    pub fn verify_proof(root: &[u8], index: usize, value: &ScalarField, proof: &[Vec<u8>]) -> bool {
        let mut current_hash = {
            let mut bytes = Vec::new();
            value.serialize_uncompressed(&mut bytes).unwrap();
            bytes
        };

        let mut current_index = index;
        for sibling in proof {
            if current_index % 2 == 0 {
                current_hash = hash(&current_hash, sibling);
            } else {
                current_hash = hash(sibling, &current_hash);
            }
            current_index /= 2;
        }

        current_hash == root
    }
}