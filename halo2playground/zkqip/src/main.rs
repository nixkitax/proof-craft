mod simple_quant;
use simple_quant::{quantize_to_fp, dequantize_from_fp};
use halo2_proofs::pasta::Fp;

fn main() {
    let scale = 1000.0;

    let values = [-3.1415, 0.0, 2.718, 5.0];
    for &v in &values {
        let q = quantize_to_fp(v, scale);
        let dq = dequantize_from_fp(q, scale);
        println!("Original: {:>7}, Quantized: {:?}, Dequantized: {:.5}", v, q, dq);
    }
}
