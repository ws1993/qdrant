use common::types::ScoreType;

use crate::data_types::vectors::{DenseVector, VectorElementTypeByte};
use crate::spaces::metric::Metric;
#[cfg(target_arch = "x86_64")]
use crate::spaces::metric_uint::avx2::dot::avx_dot_similarity_bytes;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use crate::spaces::metric_uint::neon::dot::neon_dot_similarity_bytes;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::spaces::metric_uint::sse2::dot::sse_dot_similarity_bytes;
#[cfg(target_arch = "x86_64")]
use crate::spaces::simple::MIN_DIM_SIZE_AVX;
use crate::spaces::simple::{DotProductMetric, MIN_DIM_SIZE_SIMD};
use crate::types::Distance;

impl Metric<VectorElementTypeByte> for DotProductMetric {
    fn distance() -> Distance {
        Distance::Dot
    }

    fn similarity(v1: &[VectorElementTypeByte], v2: &[VectorElementTypeByte]) -> ScoreType {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx")
                && is_x86_feature_detected!("avx2")
                && is_x86_feature_detected!("fma")
                && v1.len() >= MIN_DIM_SIZE_AVX
            {
                return unsafe { avx_dot_similarity_bytes(v1, v2) };
            }
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("sse")
                && is_x86_feature_detected!("sse2")
                && v1.len() >= MIN_DIM_SIZE_SIMD
            {
                return unsafe { sse_dot_similarity_bytes(v1, v2) };
            }
        }

        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            if std::arch::is_aarch64_feature_detected!("neon") && v1.len() >= MIN_DIM_SIZE_SIMD {
                return unsafe { neon_dot_similarity_bytes(v1, v2) };
            }
        }

        dot_similarity_bytes(v1, v2)
    }

    fn preprocess(vector: DenseVector) -> DenseVector {
        vector
    }
}

pub fn dot_similarity_bytes(
    v1: &[VectorElementTypeByte],
    v2: &[VectorElementTypeByte],
) -> ScoreType {
    let mut dot_product = 0;

    for (a, b) in v1.iter().zip(v2) {
        dot_product += i32::from(*a) * i32::from(*b);
    }

    dot_product as ScoreType
}
