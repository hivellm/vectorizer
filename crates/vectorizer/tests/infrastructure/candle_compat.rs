//! candle-core / candle-nn / candle-transformers compatibility guard.
//!
//! Pinned to verify the candle APIs our embedding pipeline actually
//! exercises across the 0.9 → 0.10 minor bump (and any future
//! Dependabot bump): the `Device` / `Tensor` / `DType` / `IndexOp` /
//! `VarBuilder` shapes consumed by `src/embedding/candle_models.rs`
//! and `src/embedding/real_models.rs`, plus a small numeric fixture
//! whose result must stay byte-identical (no f32 rounding tolerance —
//! pure CPU integer-backed ops should be deterministic).
//!
//! These tests are CPU-only on purpose: they don't need a GPU runtime
//! and run on every CI matrix entry.

#![cfg(feature = "candle-models")]

use candle_core::{DType, Device, IndexOp, Tensor};
use candle_nn::{LayerNorm, VarBuilder};
use std::collections::HashMap;

#[test]
fn dtype_set_unchanged() {
    // candle 0.10 made DType `#[non_exhaustive]`; we still depend on
    // F32 / F16 / BF16 / U32 / U8 / I64 being present. If any of
    // these go away (highly unlikely, but a non_exhaustive enum lets
    // upstream remove variants without breaking the type), the
    // embedding code stops compiling — catch it here first with a
    // direct `as u32` discriminant ref.
    let _f32 = DType::F32;
    let _f16 = DType::F16;
    let _bf16 = DType::BF16;
    let _u32 = DType::U32;
    let _u8 = DType::U8;
    let _i64 = DType::I64;
}

#[test]
fn cpu_device_constructs() {
    // `src/embedding/candle_models.rs` falls back to `Device::Cpu`
    // when CUDA is unavailable. If `Device::Cpu` ever becomes
    // gated behind a feature flag we want this to fail in CI before
    // it bricks the embedding path on Linux/macOS without GPU.
    let device = Device::Cpu;
    let scalar = Tensor::new(&[1.0_f32, 2.0, 3.0], &device).expect("create tensor");
    assert_eq!(scalar.dims(), &[3]);
    assert_eq!(scalar.dtype(), DType::F32);
}

#[test]
fn matmul_numeric_fixture() {
    // Fixed (2x3) @ (3x2) matmul. Pure CPU ops on tiny f32 inputs
    // are deterministic across versions — if this drifts the
    // numerical pipeline shifted under us.
    let device = Device::Cpu;
    let a = Tensor::new(&[[1.0_f32, 2.0, 3.0], [4.0, 5.0, 6.0]], &device).unwrap();
    let b = Tensor::new(&[[7.0_f32, 8.0], [9.0, 10.0], [11.0, 12.0]], &device).unwrap();

    let c = a.matmul(&b).expect("matmul");
    let result: Vec<Vec<f32>> = c.to_vec2().expect("to_vec2");

    // [[1*7+2*9+3*11, 1*8+2*10+3*12], [4*7+5*9+6*11, 4*8+5*10+6*12]]
    // = [[58, 64], [139, 154]]
    assert_eq!(result, vec![vec![58.0, 64.0], vec![139.0, 154.0]]);
}

#[test]
fn index_op_slicing_preserved() {
    // `IndexOp` is the trait `candle_models.rs` uses for slicing the
    // [CLS] embedding off a (batch, seq, hidden) tensor. Shape and
    // value must be preserved.
    let device = Device::Cpu;
    let t = Tensor::new(
        &[[[1.0_f32, 2.0], [3.0, 4.0]], [[5.0, 6.0], [7.0, 8.0]]],
        &device,
    )
    .unwrap();

    // Take slot 0 of the middle axis on every batch — equivalent to
    // mean-pooling onto the [CLS] token in a real BERT pipeline.
    let cls = t.i((.., 0)).expect("index op");
    assert_eq!(cls.dims(), &[2, 2]);
    assert_eq!(
        cls.to_vec2::<f32>().unwrap(),
        vec![vec![1.0, 2.0], vec![5.0, 6.0]]
    );
}

#[test]
fn var_builder_from_tensors_constructs() {
    // `RealBertEmbedding::load_safetensors` builds a `VarBuilder`
    // from a `HashMap<String, Tensor>`. The 0.10 release left this
    // signature alone — confirm.
    let device = Device::Cpu;
    let mut tensors = HashMap::new();
    tensors.insert(
        "weight".to_string(),
        Tensor::new(&[1.0_f32, 2.0, 3.0, 4.0], &device).unwrap(),
    );
    let vb = VarBuilder::from_tensors(tensors, DType::F32, &device);

    // Pull the value back out via the same shape we just inserted.
    let pulled = vb.get(4, "weight").expect("get weight");
    assert_eq!(pulled.dims(), &[4]);
    assert_eq!(pulled.dtype(), DType::F32);
}

#[test]
fn layer_norm_numeric_fixture() {
    // candle 0.10 changed layer-norm to accumulate in f32 for
    // improved precision. Our input is already f32 so the result
    // shouldn't move — assert with a narrow 1e-5 tolerance to absorb
    // any final-rounding drift but catch a real algorithm change.
    let device = Device::Cpu;
    let weight = Tensor::new(&[1.0_f32, 1.0, 1.0, 1.0], &device).unwrap();
    let bias = Tensor::new(&[0.0_f32, 0.0, 0.0, 0.0], &device).unwrap();
    let ln = LayerNorm::new(weight, bias, 1e-5);

    let input = Tensor::new(&[[1.0_f32, 2.0, 3.0, 4.0]], &device).unwrap();
    let out = candle_nn::Module::forward(&ln, &input).expect("layer norm forward");

    let result: Vec<Vec<f32>> = out.to_vec2().expect("to_vec2");
    // Mean = 2.5, var = 1.25, normalized: (x-2.5)/sqrt(1.25+eps)
    // Expected: [-1.3416, -0.4472, 0.4472, 1.3416] (within 1e-5)
    let expected = [-1.341_640_8_f32, -0.447_213_6, 0.447_213_6, 1.341_640_8];
    assert_eq!(result.len(), 1);
    for (i, (actual, expected)) in result[0].iter().zip(expected.iter()).enumerate() {
        let diff = (actual - expected).abs();
        assert!(
            diff < 1e-5,
            "layer-norm output drifted at index {i}: got {actual}, expected {expected}, diff {diff}",
        );
    }
}
