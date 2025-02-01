const NOT_QUITE_ONE: f64 = 0.9999999999f64;

/// Defines the algorithm used when a random individual is needed from a pool of individuals that has been sorted by a
/// fitness function. The sorting algorithm defines the greatest fitness as being sorted at the end of a vector where
/// `pool.sort_by(fitness_fn)` has been called.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SelectionCurve {
    // All individuals are as likely as any other to be selected
    Fair,

    // The fitter individuals will appear much more often
    StrongPreferenceForFit,

    // The fitter individuals will appear more often
    PreferenceForFit,

    // The fitter individuals will appear a little more often
    SlightPreferenceForFit,

    // The less fit individuals will appear a little more often
    SlightPreferenceForUnfit,

    // The less fit individuals will appear more often
    PreferenceForUnfit,

    // The less fit individuals will appear much more often
    StrongPreferenceForUnfit,
}

impl SelectionCurve {
    /// Randomly selects a value in the range [0 .. number_of_individuals] according to the SelectionCurve properties
    pub fn pick_one_index<R: rand::Rng>(&self, rng: &mut R, number_of_individuals: usize) -> usize {
        // Pick a value in the range of (0.0 .. 1.0] (includes zero, but not one). This behavior is part of the
        // guarantee of the rand::distributions::Standard spec
        let pick: f64 = rng.random();

        // Use exponential scaling for the preferences
        let pick = match &self {
            SelectionCurve::Fair => pick,
            SelectionCurve::SlightPreferenceForFit | SelectionCurve::SlightPreferenceForUnfit => {
                pick * pick
            }
            SelectionCurve::PreferenceForFit | SelectionCurve::PreferenceForUnfit => {
                pick * pick * pick
            }
            SelectionCurve::StrongPreferenceForFit | SelectionCurve::StrongPreferenceForUnfit => {
                pick * pick * pick * pick * pick * pick
            }
        };

        // Reverse the direction of the 'Fit' selection
        let pick = match &self {
            SelectionCurve::PreferenceForFit
            | SelectionCurve::SlightPreferenceForFit
            | SelectionCurve::StrongPreferenceForFit => 1.0 - pick,
            _ => pick,
        };

        // With rounding error, it's possible to get a pick that's >= 1.0, so turn that into 0.9999999
        let pick = if pick >= 1.0 { NOT_QUITE_ONE } else { pick };

        // Multiply the pick by the number of individuals and turn it into an integer
        (pick * number_of_individuals as f64).floor() as usize
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::SmallRng; // cspell:disable-line
    use rand::SeedableRng;

    use crate::*;

    fn pick_100_000_times(curve: SelectionCurve) -> Vec<usize> {
        let mut rng = SmallRng::seed_from_u64(1234);
        let mut buckets = vec![0usize; 100];

        // Pick from 0 to 100, 100_000 times
        for _ in 0..100_000 {
            let pick = curve.pick_one_index(&mut rng, 100);
            buckets[pick] += 1;
        }

        buckets
    }

    #[test]
    fn fair_selection_curve() {
        let buckets = pick_100_000_times(SelectionCurve::Fair);

        // Each bucket should have at least 900 and no more than 1100
        for (i, &bucket) in buckets.iter().enumerate() {
            assert!(
                bucket >= 900 && bucket <= 1100,
                "bucket[{}] had {}",
                i,
                bucket
            );
        }
    }

    #[test]
    fn slight_preference_selection_curve() {
        let buckets = pick_100_000_times(SelectionCurve::SlightPreferenceForFit);

        // Each bucket should be no more than 100 less than the previous bucket. Also, we should not get over 600 entries
        // until at least 1/5 of the way through
        let mut last_bucket_count = 0;
        for (i, &bucket) in buckets.iter().enumerate() {
            assert!(
                bucket + 100 >= last_bucket_count,
                "bucket[{}] was {}, but the previous bucket held {}",
                i,
                bucket,
                last_bucket_count
            );
            if i < 20 {
                assert!(
                    bucket <= 600,
                    "bucket[{}] had {} but should have had less than 600",
                    i,
                    bucket
                );
            }

            last_bucket_count = bucket;
        }

        // The unfit bucket has the same test, with the order of picks reversed
        let mut buckets = pick_100_000_times(SelectionCurve::SlightPreferenceForUnfit);
        buckets.reverse();

        // Each bucket should be no more than 100 less than the previous bucket. Also, we should not get over 600 entries
        // until at least 1/5 of the way through
        let mut last_bucket_count = 0;
        for (i, &bucket) in buckets.iter().enumerate() {
            assert!(
                bucket + 100 >= last_bucket_count,
                "bucket[{}] was {}, but the previous bucket held {}",
                i,
                bucket,
                last_bucket_count
            );
            if i < 20 {
                assert!(
                    bucket <= 600,
                    "bucket[{}] had {} but should have had less than 600",
                    i,
                    bucket
                );
            }

            last_bucket_count = bucket;
        }
    }

    #[test]
    fn preference_selection_curve() {
        let buckets = pick_100_000_times(SelectionCurve::PreferenceForFit);

        // Each bucket should be no more than 100 less than the previous bucket. Also, we should not get over 600 entries
        // until at least 1/2 of the way through
        let mut last_bucket_count = 0;
        for (i, &bucket) in buckets.iter().enumerate() {
            assert!(
                bucket + 100 >= last_bucket_count,
                "bucket[{}] was {}, but the previous bucket held {}",
                i,
                bucket,
                last_bucket_count
            );
            if i < 50 {
                assert!(
                    bucket <= 600,
                    "bucket[{}] had {} but should have had less than 600",
                    i,
                    bucket
                );
            }

            last_bucket_count = bucket;
        }

        // The unfit bucket has the same test, with the order of picks reversed
        let mut buckets = pick_100_000_times(SelectionCurve::PreferenceForUnfit);
        buckets.reverse();

        // Each bucket should be no more than 100 less than the previous bucket. Also, we should not get over 600 entries
        // until at least 1/2 of the way through
        let mut last_bucket_count = 0;
        for (i, &bucket) in buckets.iter().enumerate() {
            assert!(
                bucket + 100 >= last_bucket_count,
                "bucket[{}] was {}, but the previous bucket held {}",
                i,
                bucket,
                last_bucket_count
            );
            if i < 50 {
                assert!(
                    bucket <= 600,
                    "bucket[{}] had {} but should have had less than 600",
                    i,
                    bucket
                );
            }

            last_bucket_count = bucket;
        }
    }

    #[test]
    fn strong_preference_selection_curve() {
        let buckets = pick_100_000_times(SelectionCurve::StrongPreferenceForFit);

        // Each bucket should be no more than 100 less than the previous bucket. Also, we should not get over 600 entries
        // until at least 3/4 of the way through
        let mut last_bucket_count = 0;
        for (i, &bucket) in buckets.iter().enumerate() {
            assert!(
                bucket + 100 >= last_bucket_count,
                "bucket[{}] was {}, but the previous bucket held {}",
                i,
                bucket,
                last_bucket_count
            );
            if i < 75 {
                assert!(
                    bucket <= 600,
                    "bucket[{}] had {} but should have had less than 600",
                    i,
                    bucket
                );
            }

            last_bucket_count = bucket;
        }

        // The unfit bucket has the same test, with the order of picks reversed
        let mut buckets = pick_100_000_times(SelectionCurve::StrongPreferenceForUnfit);
        buckets.reverse();

        // Each bucket should be no more than 100 less than the previous bucket. Also, we should not get over 600 entries
        // until at least 3/4 of the way through
        let mut last_bucket_count = 0;
        for (i, &bucket) in buckets.iter().enumerate() {
            assert!(
                bucket + 100 >= last_bucket_count,
                "bucket[{}] was {}, but the previous bucket held {}",
                i,
                bucket,
                last_bucket_count
            );
            if i < 75 {
                assert!(
                    bucket <= 600,
                    "bucket[{}] had {} but should have had less than 600",
                    i,
                    bucket
                );
            }

            last_bucket_count = bucket;
        }
    }
}
