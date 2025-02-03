use common::math::scaled_fast_sigmoid;
use common::types::ScoreType;
use itertools::Itertools;

use super::{Query, TransformInto};
use crate::common::operation_error::OperationResult;
use crate::data_types::vectors::{QueryVector, VectorInternal};

#[derive(Debug, Clone, PartialEq)]
pub struct RecoQuery<T> {
    pub positives: Vec<T>,
    pub negatives: Vec<T>,
}

impl<T> RecoQuery<T> {
    pub fn new(positives: Vec<T>, negatives: Vec<T>) -> Self {
        Self {
            positives,
            negatives,
        }
    }

    pub fn flat_iter(&self) -> impl Iterator<Item = &T> {
        self.positives.iter().chain(self.negatives.iter())
    }
}

impl<T, U> TransformInto<RecoQuery<U>, T, U> for RecoQuery<T> {
    fn transform<F>(self, mut f: F) -> OperationResult<RecoQuery<U>>
    where
        F: FnMut(T) -> OperationResult<U>,
    {
        Ok(RecoQuery::new(
            self.positives.into_iter().map(&mut f).try_collect()?,
            self.negatives.into_iter().map(&mut f).try_collect()?,
        ))
    }
}

impl<T> Query<T> for RecoQuery<T> {
    fn score_by(&self, similarity: impl Fn(&T) -> ScoreType) -> ScoreType {
        // get similarities to all positives
        let positive_similarities = self.positives.iter().map(&similarity);

        // and all negatives
        let negative_similarities = self.negatives.iter().map(&similarity);

        merge_similarities(positive_similarities, negative_similarities)
    }
}

#[inline]
fn merge_similarities(
    positives: impl Iterator<Item = ScoreType>,
    negatives: impl Iterator<Item = ScoreType>,
) -> ScoreType {
    // get max similarity to positives and max to negatives
    let max_positive = positives
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(ScoreType::NEG_INFINITY);

    let max_negative = negatives
        .max_by(|a, b| a.total_cmp(b))
        .unwrap_or(ScoreType::NEG_INFINITY);

    if max_positive > max_negative {
        scaled_fast_sigmoid(max_positive)
    } else {
        -scaled_fast_sigmoid(max_negative)
    }
}

impl From<RecoQuery<VectorInternal>> for QueryVector {
    fn from(query: RecoQuery<VectorInternal>) -> Self {
        QueryVector::Recommend(query)
    }
}

#[cfg(test)]
mod test {
    use std::cmp::Ordering;

    use common::math::scaled_fast_sigmoid;
    use common::types::ScoreType;
    use proptest::prelude::*;
    use rstest::rstest;

    use super::RecoQuery;
    use crate::vector_storage::query::Query;

    enum Chosen {
        Positive,
        Negative,
    }

    #[rstest]
    #[case::higher_positive(vec![42], vec![4], Chosen::Positive, 42.0)]
    #[case::higher_negative(vec![4], vec![42], Chosen::Negative, 42.0)]
    #[case::negative_zero(vec![-1], vec![0], Chosen::Negative, 0.0)]
    #[case::positive_zero(vec![0], vec![-1], Chosen::Positive, 0.0)]
    #[case::both_under_zero(vec![-42], vec![-84], Chosen::Positive, -42.0)]
    #[case::both_under_zero_but_negative_is_higher(vec![-84], vec![-42], Chosen::Negative, -42.0)]
    #[case::multiple_with_negative_best(vec![1, 2, 3], vec![4, 5, 6], Chosen::Negative, 6.0)]
    #[case::multiple_with_positive_best(vec![10, 2, 3], vec![4, 5, 6], Chosen::Positive, 10.0)]
    fn score_query(
        #[case] positives: Vec<isize>,
        #[case] negatives: Vec<isize>,
        #[case] chosen: Chosen,
        #[case] expected: ScoreType,
    ) {
        let query = RecoQuery::new(positives, negatives);

        let dummy_similarity = |x: &isize| *x as ScoreType;

        let positive_transformation = scaled_fast_sigmoid;
        let negative_transformation = |x| -scaled_fast_sigmoid(x);

        let score = query.score_by(dummy_similarity);

        match chosen {
            Chosen::Positive => {
                assert_eq!(score, positive_transformation(expected));
            }
            Chosen::Negative => {
                assert_eq!(score, negative_transformation(expected));
            }
        }
    }

    fn ulps_eq(a: f32, b: f32, ulps: u32) -> bool {
        if a.signum() != b.signum() {
            return false;
        }

        let a = a.to_bits();
        let b = b.to_bits();

        a.abs_diff(b) <= ulps
    }

    /// Relaxes the comparison of floats to allow for a some difference in units of least precision
    fn float_cmp(a: f32, b: f32) -> Ordering {
        if ulps_eq(a, b, 80) {
            Ordering::Equal
        } else {
            a.total_cmp(&b)
        }
    }

    proptest! {
        /// Checks that the negative-chosen scores invert the order of the candidates
        #[test]
        fn correct_negative_order(a in -100f32..=100f32, b in -100f32..=100f32) {
            let dummy_similarity = |x: &f32| *x as ScoreType;

            let ordering_before = float_cmp(dummy_similarity(&a), dummy_similarity(&b));

            let query_a = RecoQuery::new(vec![], vec![a]);
            let query_b = RecoQuery::new(vec![], vec![b]);

            let score_a = query_a.score_by(dummy_similarity);
            let score_b = query_b.score_by(dummy_similarity);

            let ordering_after = float_cmp(score_a, score_b);

            if ordering_before == std::cmp::Ordering::Equal {
                assert_eq!(ordering_before, ordering_after);
            } else {
                assert_ne!(ordering_before, ordering_after)
            }
        }

        /// Checks that the positive-chosen scores preserve the order of the candidates
        #[test]
        fn correct_positive_order(a in -100f32..=100f32, b in -100f32..=100f32) {
            let dummy_similarity = |x: &f32| *x as ScoreType;

            let ordering_before = float_cmp(dummy_similarity(&a), dummy_similarity(&b));

            // Too similar scores can get compressed to the same value by the sigmoid function.
            // This would make the test useless, so we skip those cases.
            prop_assume!(ordering_before != Ordering::Equal);

            let query_a = RecoQuery::new(vec![a], vec![]);
            let query_b = RecoQuery::new(vec![b], vec![]);

            let score_a = query_a.score_by(dummy_similarity);
            let score_b = query_b.score_by(dummy_similarity);

            let ordering_after = score_a.total_cmp(&score_b);

            assert_eq!(ordering_before, ordering_after);
        }

        /// Guarantees that the point that was chosen from positive is always preferred on
        /// the candidate list over a point that was chosen from negatives
        #[test]
        fn correct_positive_and_negative_order(p in -100f32..=100f32, n in -100f32..=100f32) {
            let dummy_similarity = |x: &f32| *x as ScoreType;

            let query_p = RecoQuery::new(vec![p], vec![]);
            let query_n = RecoQuery::new(vec![], vec![n]);

            let ordering = query_p.score_by(dummy_similarity).total_cmp(&query_n.score_by(dummy_similarity));

            assert_ne!(ordering, std::cmp::Ordering::Less);
        }
    }
}
