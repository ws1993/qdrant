use common::types::ScoreType;

use crate::data_types::vectors::{DenseVector, VectorElementTypeByte};
use crate::spaces::metric::Metric;
#[cfg(target_arch = "x86_64")]
use crate::spaces::metric_uint::avx2::euclid::avx_euclid_similarity_bytes;
#[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
use crate::spaces::metric_uint::neon::euclid::neon_euclid_similarity_bytes;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
use crate::spaces::metric_uint::sse2::euclid::sse_euclid_similarity_bytes;
#[cfg(target_arch = "x86_64")]
use crate::spaces::simple::MIN_DIM_SIZE_AVX;
use crate::spaces::simple::{EuclidMetric, MIN_DIM_SIZE_SIMD};
use crate::types::Distance;

impl Metric<VectorElementTypeByte> for EuclidMetric {
    fn distance() -> Distance {
        Distance::Euclid
    }

    fn similarity(v1: &[VectorElementTypeByte], v2: &[VectorElementTypeByte]) -> ScoreType {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx")
                && is_x86_feature_detected!("avx2")
                && is_x86_feature_detected!("fma")
                && v1.len() >= MIN_DIM_SIZE_AVX
            {
                return unsafe { avx_euclid_similarity_bytes(v1, v2) };
            }
        }

        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            if is_x86_feature_detected!("sse")
                && is_x86_feature_detected!("sse2")
                && v1.len() >= MIN_DIM_SIZE_SIMD
            {
                return unsafe { sse_euclid_similarity_bytes(v1, v2) };
            }
        }

        #[cfg(all(target_arch = "aarch64", target_feature = "neon"))]
        {
            if std::arch::is_aarch64_feature_detected!("neon") && v1.len() >= MIN_DIM_SIZE_SIMD {
                return unsafe { neon_euclid_similarity_bytes(v1, v2) };
            }
        }

        euclid_similarity_bytes(v1, v2)
    }

    fn preprocess(vector: DenseVector) -> DenseVector {
        vector
    }
}

pub fn euclid_similarity_bytes(
    v1: &[VectorElementTypeByte],
    v2: &[VectorElementTypeByte],
) -> ScoreType {
    -v1.iter()
        .zip(v2)
        .map(|(a, b)| {
            let diff = i32::from(*a) - i32::from(*b);
            diff * diff
        })
        .sum::<i32>() as ScoreType
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use super::*;
    use crate::data_types::primitive::PrimitiveVectorElement;
    use crate::data_types::vectors::{TypedDenseVector, VectorElementType};

    #[test]
    fn test_conversion_to_bytes() {
        let dense_vector = DenseVector::from(vec![-10.0, 1.0, 2.0, 3.0, 255., 300.]);
        let preprocessed_vector =
            <EuclidMetric as Metric<VectorElementType>>::preprocess(dense_vector);
        let typed_dense_vector =
            VectorElementTypeByte::slice_from_float_cow(Cow::from(preprocessed_vector));
        let expected: TypedDenseVector<VectorElementTypeByte> = vec![0, 1, 2, 3, 255, 255];
        assert_eq!(typed_dense_vector, expected);
    }
}
