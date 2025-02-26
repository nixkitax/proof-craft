use criterion::{criterion_group, criterion_main, Criterion};
use merkletree::merkle_tree::MerkleTree;
use ark_bls12_381::Fr as ScalarField;
use ark_ff::UniformRand;

fn bench_merkle_tree_construction(c: &mut Criterion) {
    let mut rng = ark_std::test_rng();
    let values: Vec<ScalarField> = (0..1000).map(|_| ScalarField::rand(&mut rng)).collect();

    c.bench_function("Merkle Tree Construction (1000 leaves)", |b| {
        b.iter(|| MerkleTree::new(values.clone()))
    });
}

fn bench_merkle_proof_generation(c: &mut Criterion) {
    let mut rng = ark_std::test_rng();
    let values: Vec<ScalarField> = (0..1000).map(|_| ScalarField::rand(&mut rng)).collect();
    let tree = MerkleTree::new(values.clone());

    c.bench_function("Merkle Proof Generation (1000 leaves)", |b| {
        b.iter(|| tree.generate_proof(500)) // Genera una prova per l'indice 500
    });
}

fn bench_merkle_proof_verification(c: &mut Criterion) {
    let mut rng = ark_std::test_rng();
    let values: Vec<ScalarField> = (0..1000).map(|_| ScalarField::rand(&mut rng)).collect();
    let tree = MerkleTree::new(values.clone());
    let proof = tree.generate_proof(500); // Genera una prova per l'indice 500

    c.bench_function("Merkle Proof Verification (1000 leaves)", |b| {
        b.iter(|| MerkleTree::verify_proof(tree.root(), 500, &values[500], &proof))
    });
}

criterion_group!(
    benches,
    bench_merkle_tree_construction,
    bench_merkle_proof_generation,
    bench_merkle_proof_verification 
);
criterion_main!(benches);
