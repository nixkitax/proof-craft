use sha2::{Sha256, Digest};

// Computes the SHA-256 hash of two byte slices concatenated together.
pub fn hash(left: &[u8], right: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(left);
    hasher.update(right);
    hasher.finalize().to_vec()
}