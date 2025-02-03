use common::types::ScoreType;

use crate::data_types::vectors::{DenseVector, VectorElementTypeByte};
use crate::spaces::metric::Metric;
#[cfg(target_arch = "x86_64")]
use crate::spaces::metric_uint::avx2::cosine::avx_cosine_similarity_bytes;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use crate::spaces::metric_uint::neon::cosine::neon_cosine_similarity_bytes;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::spaces::metric_uint::sse2::cosine::sse_cosine_similarity_bytes;
#[cfg(target_arch = "x86_64")]
use crate::spaces::simple::MIN_DIM_SIZE_AVX;
use crate::spaces::simple::{CosineMetric, MIN_DIM_SIZE_SIMD};
use crate::types::Distance;

impl Metric<VectorElementTypeByte> for CosineMetric {
    fn distance() -> Distance {
        Distance::Cosine
    }

    fn similarity(v1: &[VectorElementTypeByte], v2: &[VectorElementTypeByte]) -> ScoreType {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx")
                && is_x86_feature_detected!("avx2")
                && is_x86_feature_detected!("fma")
                && v1.len() >= MIN_DIM_SIZE_AVX
            {
                return unsafe { avx_cosine_similarity_bytes(v1, v2) };
            }
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("sse")
                && is_x86_feature_detected!("sse2")
                && v1.len() >= MIN_DIM_SIZE_SIMD
            {
                return unsafe { sse_cosine_similarity_bytes(v1, v2) };
            }
        }

        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            if std::arch::is_aarch64_feature_detected!("neon") && v1.len() >= MIN_DIM_SIZE_SIMD {
                return unsafe { neon_cosine_similarity_bytes(v1, v2) };
            }
        }

        cosine_similarity_bytes(v1, v2)
    }

    fn preprocess(vector: DenseVector) -> DenseVector {
        vector
    }
}

pub fn cosine_similarity_bytes(
    v1: &[VectorElementTypeByte],
    v2: &[VectorElementTypeByte],
) -> ScoreType {
    let mut dot_product = 0;
    let mut norm1 = 0;
    let mut norm2 = 0;

    for (a, b) in v1.iter().zip(v2) {
        dot_product += i32::from(*a) * i32::from(*b);
        norm1 += i32::from(*a) * i32::from(*a);
        norm2 += i32::from(*b) * i32::from(*b);
    }

    if norm1 == 0 || norm2 == 0 {
        return 0.0;
    }

    dot_product as ScoreType / (norm1 as ScoreType * norm2 as ScoreType).sqrt()
}

#[test]
fn test_zero() {
    let v1: Vec<u8> = vec![0, 0, 0, 0, 0, 0, 0, 0];
    let v2: Vec<u8> = vec![255, 255, 0, 254, 253, 252, 251, 250];

    assert_eq!(cosine_similarity_bytes(&v1, &v2), 0.0);
    assert_eq!(cosine_similarity_bytes(&v2, &v1), 0.0);
    assert_eq!(cosine_similarity_bytes(&v1, &v1), 0.0);
}
