//! Modulo per quantizzazione/dequantizzazione semplice f64 <-> campo Fp (Halo2).

use halo2_proofs::pasta::Fp;
use ff::PrimeField;

/// Quantizza un valore `f64` in un campo `Fp` con un dato fattore di scala.
pub fn quantize_to_fp(value: f64, scale: f64) -> Fp {
    let scaled = (value * scale).round() as i128;
    if scaled >= 0 {
        Fp::from_u128(scaled as u128)
    } else {
        -Fp::from_u128((-scaled) as u128)
    }
}

/// Dequantizza un `Fp` in un `f64` con lo stesso fattore di scala usato in `quantize_to_fp`.
pub fn dequantize_from_fp(elem: Fp, scale: f64) -> f64 {
    let repr = elem.to_repr();
    let lower_bytes: [u8; 16] = repr[0..16].try_into().unwrap();
    let upper_bytes: [u8; 16] = repr[16..32].try_into().unwrap();
    let value_lower = u128::from_le_bytes(lower_bytes);
    let value_upper = u128::from_le_bytes(upper_bytes);

    // Parametri del campo Fp di Pasta (BN254 scalar field)
    let mod_hex = &Fp::MODULUS[2..]; // rimuove "0x"
    let (mod_upper_hex, mod_lower_hex) = mod_hex.split_at(32);
    let mod_upper = u128::from_str_radix(mod_upper_hex, 16).unwrap();
    let mod_lower = u128::from_str_radix(mod_lower_hex, 16).unwrap();
    let carry = (mod_upper & 1) << 127;
    let half_upper = mod_upper >> 1;
    let half_lower = (mod_lower >> 1) | carry;

    let is_negative = match value_upper.cmp(&half_upper) {
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => value_lower > half_lower,
    };

    let signed: i128 = if is_negative {
        // modulo - value
        let (diff_lower, borrow) = if value_lower > mod_lower {
            (mod_lower.wrapping_add(1u128 << 127).wrapping_sub(value_lower), 1)
        } else {
            (mod_lower - value_lower, 0)
        };
        let _ = mod_upper - value_upper - borrow;
        -(diff_lower as i128)
    } else {
        value_lower as i128
    };

    signed as f64 / scale
}
