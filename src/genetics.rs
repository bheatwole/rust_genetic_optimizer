use rand::rngs::StdRng; // cspell:disable-line

pub trait Genetics {
    /// Produces a random individual of up to the `max_points` number of code items.
    fn random_individual(&self, rng: &mut StdRng, max_points: usize) -> u64;

    /// Mutates the given individual by replacing `points` number of code items with new random code.
    fn mutate(&self, rng: &mut StdRng, individual: u64, points: usize) -> u64;

    /// Combines the code of two individuals by swapping `points` number of code items between them.
    fn crossover(
        &self,
        rng: &mut StdRng,
        individual_a: u64,
        individual_b: u64,
        points: usize,
    ) -> u64;
}
