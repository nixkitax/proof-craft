use criterion::{criterion_group, criterion_main, Criterion};
use zkqip::{quantize_to_fp, dequantize_fp, inner_product_fp};

pub fn bench_quantize(c: &mut Criterion) {
    println!("Running quantize benchmark..."); // Debug print

    let value = -2.718;
    let scale = 32;

    c.bench_function("quantize_to_fp", |b| {
        b.iter(|| {
            let _ = quantize_to_fp(value, scale);
        });
    });
}

pub fn bench_dequantize(c: &mut Criterion) {
    println!("Running dequantize benchmark..."); // Debug print

    let value = -2.718;
    let scale = 32;
    let q = quantize_to_fp(value, scale);

    c.bench_function("dequantize_fp", |b| {
        b.iter(|| {
            let _ = dequantize_fp(q, scale);
        });
    });
}

fn bench_inner_product(c: &mut Criterion) {
    let precision_bits = 32;

    // Vettori grandi per il benchmark
    let v1_real: Vec<f64> = (0..1000).map(|x| x as f64 * 0.01).collect();
    let v2_real: Vec<f64> = (0..1000).map(|x| (x as f64 * 0.02) - 5.0).collect();

    let v1_q: Vec<_> = v1_real.iter().map(|&x| quantize_to_fp(x, precision_bits)).collect();
    let v2_q: Vec<_> = v2_real.iter().map(|&x| quantize_to_fp(x, precision_bits)).collect();

    c.bench_function("inner_product_fp", |b| {
        b.iter(|| {
            let _ = inner_product_fp(&v1_q, &v2_q, precision_bits);
        });
    });
}
criterion_group!(benches, bench_quantize, bench_dequantize, bench_inner_product);
criterion_main!(benches);
