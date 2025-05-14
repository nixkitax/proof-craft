use zkqip::{quantize_to_fp, dequantize_fp, SimpleQuantCircuit};
use halo2_proofs::dev::MockProver;

#[test]
fn test_quantization_round_trip() {
    let value = -2.718;
    let scale = 32;
    let quantized = quantize_to_fp(value, scale);
    let dequantized = dequantize_fp(quantized, scale);
    let diff = (value - dequantized).abs();

    assert!(diff < 1e-6, "Dequantized value {} differs too much from {}", dequantized, value);
}

#[test]
fn test_quantized_addition_correct() {
    let a = 1.5;
    let b = 2.5;
    let scale = 32;
    let expected_sum = a + b;

    let circuit = SimpleQuantCircuit { a, b, expected_sum, scale };
    let k = 4;
    let prover = MockProver::run(k, &circuit, vec![]).unwrap();
    prover.assert_satisfied();
}
