//! Numerical-parity oracle for the phase-7e primitives.
//!
//! Every new op (`normalize_in_place`, `manhattan_distance`,
//! `add_assign`, `sub_assign`, `scale`, `horizontal_min_index`)
//! must agree with the [`vectorizer::simd::scalar::ScalarBackend`]
//! within f32 rounding on a battery of random vectors. Same shape
//! as `tests/simd/scalar_oracle.rs` for the original four ops.
//!
//! If any backend overrides one of these primitives (AVX2 overrides
//! `manhattan_distance`, NEON does the same), this test catches a
//! divergence between the SIMD path and the scalar oracle.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use vectorizer::simd::backend::SimdBackend;
use vectorizer::simd::scalar::ScalarBackend;
use vectorizer::simd::{
    add_assign, dequantize_u8_to_f32, horizontal_min_index, l2_norm, manhattan_distance,
    normalize_in_place, quantize_f32_to_u8, scale, sub_assign,
};

/// Linear congruential generator — same one the original oracle uses
/// so the seed values stay comparable across the two test files.
fn lcg(state: &mut u64) -> f32 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let bits = (*state >> 40) as u32;
    let unit = (bits as f32) / ((1u32 << 24) as f32);
    unit * 2.0 - 1.0
}

fn random_vector(seed: u64, len: usize) -> Vec<f32> {
    let mut state = seed;
    (0..len).map(|_| lcg(&mut state)).collect()
}

fn tolerance(len: usize) -> f32 {
    // Loosened slightly vs. dot-product because Manhattan
    // accumulates the absolute values directly (no cancellation
    // helping the reduction), and the result magnitude scales with
    // `len` so even relative-tolerance comparisons benefit from a
    // proportional bound. Worst-case rounding error for an `n`-
    // element non-cancelling reduction is `eps * n` — we use that
    // plus a safety factor of 8.
    f32::EPSILON * (len as f32) * 8.0
}

#[test]
fn manhattan_distance_matches_oracle() {
    let scalar = ScalarBackend;
    for &len in &[5usize, 8, 13, 128, 256, 999, 1024] {
        let a = random_vector(0x1234_5678 ^ len as u64, len);
        let b = random_vector(0x8765_4321 ^ len as u64, len);
        let got = manhattan_distance(&a, &b);
        let want = scalar.manhattan_distance(&a, &b);
        let tol = tolerance(len);
        assert!(
            (got - want).abs() <= tol,
            "len={len} got={got} want={want} tol={tol}"
        );
    }
}

#[test]
fn normalize_in_place_produces_unit_vector() {
    for &len in &[8usize, 64, 384, 1024] {
        let mut a = random_vector(0xAAAA_BBBB ^ len as u64, len);
        normalize_in_place(&mut a);
        let norm = l2_norm(&a);
        // Tolerance is generous because the reduction + divide both
        // accumulate rounding error.
        assert!(
            (norm - 1.0).abs() < 1e-5,
            "len={len}: post-normalise norm = {norm} (expected ~1.0)"
        );
    }
}

#[test]
fn normalize_in_place_zero_vector_is_noop() {
    let mut a = vec![0.0f32; 16];
    normalize_in_place(&mut a);
    // Every element stays exactly zero — no NaN propagation.
    for &x in &a {
        assert_eq!(x, 0.0, "zero vector should be unchanged");
    }
}

#[test]
fn add_assign_matches_oracle() {
    let scalar = ScalarBackend;
    for &len in &[5usize, 8, 128, 1024] {
        let mut a = random_vector(0xCCCC_1111 ^ len as u64, len);
        let b = random_vector(0xDDDD_2222 ^ len as u64, len);
        let mut want = a.clone();
        scalar.add_assign(&mut want, &b);
        add_assign(&mut a, &b);
        for (got, exp) in a.iter().zip(want.iter()) {
            assert!((got - exp).abs() < 1e-5, "got={got} want={exp}");
        }
    }
}

#[test]
fn sub_assign_matches_oracle() {
    let scalar = ScalarBackend;
    for &len in &[5usize, 8, 128, 1024] {
        let mut a = random_vector(0xEEEE_3333 ^ len as u64, len);
        let b = random_vector(0xFFFF_4444 ^ len as u64, len);
        let mut want = a.clone();
        scalar.sub_assign(&mut want, &b);
        sub_assign(&mut a, &b);
        for (got, exp) in a.iter().zip(want.iter()) {
            assert!((got - exp).abs() < 1e-5, "got={got} want={exp}");
        }
    }
}

#[test]
fn scale_matches_oracle() {
    let scalar = ScalarBackend;
    for &len in &[5usize, 8, 128, 1024] {
        let mut a = random_vector(0x5555_6666 ^ len as u64, len);
        let mut want = a.clone();
        let s = 0.7;
        scalar.scale(&mut want, s);
        scale(&mut a, s);
        for (got, exp) in a.iter().zip(want.iter()) {
            assert!((got - exp).abs() < 1e-5, "got={got} want={exp}");
        }
    }
}

#[test]
fn horizontal_min_index_finds_smallest() {
    // Hand-picked input: minimum at index 3.
    let a = [3.0f32, 1.5, 2.7, 0.1, 4.2, 1.0];
    let got = horizontal_min_index(&a);
    assert_eq!(got, Some((3, 0.1)));
}

#[test]
fn horizontal_min_index_empty_returns_none() {
    let a: [f32; 0] = [];
    assert_eq!(horizontal_min_index(&a), None);
}

#[test]
fn horizontal_min_index_handles_singletons() {
    let a = [42.0f32];
    assert_eq!(horizontal_min_index(&a), Some((0, 42.0)));
}

#[test]
fn horizontal_min_index_first_occurrence_wins_on_tie() {
    let a = [1.0f32, 0.5, 0.5, 0.5, 2.0];
    let got = horizontal_min_index(&a);
    // Every backend returns the FIRST occurrence of the minimum to
    // keep the result deterministic.
    assert_eq!(got, Some((1, 0.5)));
}

// ── phase 7f: quantize / dequantize parity tests ────────────────────

#[test]
fn quantize_f32_to_u8_round_trip_within_one_step() {
    // Pre-7f scalar reference (`quantize_8bit`):
    //   normalised = (v - offset) / scale
    //   clamped    = normalised.clamp(0, levels - 1)
    //   code       = clamped.round() as u8
    // Trait default (and any SIMD override) must round-trip within
    // one quantization step of the original.
    for &len in &[5usize, 8, 33, 256, 1024] {
        let src = random_vector(0xABCD_0000 ^ len as u64, len);
        // Pick scale/offset so the input range maps onto [0, 255].
        let offset = -1.0f32;
        let scale = 2.0 / 255.0; // input ∈ [-1, 1] → codes ∈ [0, 255]
        let mut codes = vec![0u8; len];
        let mut decoded = vec![0.0f32; len];
        quantize_f32_to_u8(&src, &mut codes, scale, offset, 256);
        dequantize_u8_to_f32(&codes, &mut decoded, scale, offset);
        for (orig, back) in src.iter().zip(decoded.iter()) {
            // Each code represents a `scale`-wide bucket; round-trip
            // error is bounded by `scale` (one step) plus a tiny
            // f32 rounding margin.
            let err = (orig - back).abs();
            assert!(
                err <= scale + 1e-6,
                "len={len}: orig={orig} back={back} err={err}"
            );
        }
    }
}

#[test]
fn quantize_f32_to_u8_clamps_out_of_range_input() {
    // Inputs outside [offset, offset + (levels-1)*scale] get clamped
    // to the end codes — never wrap around.
    let src = vec![-100.0f32, 100.0, 0.0, 0.5];
    let mut codes = vec![0u8; src.len()];
    quantize_f32_to_u8(&src, &mut codes, 1.0 / 255.0, 0.0, 256);
    assert_eq!(codes[0], 0, "huge negative should clamp to 0");
    assert_eq!(codes[1], 255, "huge positive should clamp to 255");
    // 0.0 / scale = 0, rounded = 0
    assert_eq!(codes[2], 0);
    // 0.5 / (1/255) = 127.5; round-half-to-even gives either 127 or
    // 128 depending on f32 rounding — both are acceptable.
    assert!(
        (i32::from(codes[3]) - 128).abs() <= 1,
        "0.5 should land near the midpoint, got {}",
        codes[3]
    );
}

#[test]
fn quantize_f32_to_u8_constant_input_writes_zeros() {
    // A constant-valued dataset gives `scale == 0.0` (max == min).
    // The SIMD primitive must handle this without panicking — the
    // hnsw_integration::test_quantization_stats regression that
    // surfaced after phase 7f used exactly this fixture.
    let src = vec![1.0f32; 100];
    let mut codes = vec![0u8; src.len()];
    quantize_f32_to_u8(&src, &mut codes, 0.0, 1.0, 256);
    // Every code must be 0 — there's no information to encode when
    // every input is identical.
    for (i, &c) in codes.iter().enumerate() {
        assert_eq!(c, 0, "code[{i}] = {c}, expected 0 for constant input");
    }
}

#[test]
fn dequantize_u8_to_f32_is_linear() {
    // Dequantize is the trivial linear map: `offset + code * scale`.
    let codes = [0u8, 64, 128, 255];
    let mut out = vec![0.0f32; codes.len()];
    let scale = 1.0 / 255.0;
    let offset = -0.5;
    dequantize_u8_to_f32(&codes, &mut out, scale, offset);
    for (i, &c) in codes.iter().enumerate() {
        let expected = offset + f32::from(c) * scale;
        assert!((out[i] - expected).abs() < 1e-6);
    }
}
