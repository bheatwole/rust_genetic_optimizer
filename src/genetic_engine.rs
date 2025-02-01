use crate::{GeneticEngineBuilder, GeneticError, Genetics};
use rand::Rng;
use rand::{rngs::StdRng, SeedableRng}; // cspell:disable-line

pub struct GeneticEngine<G>
where
    G: Genetics,
{
    rng: StdRng,
    mutation_rate: u8,
    crossover_rate: u8,
    max_mutation_points: u8,
    max_crossover_points: u8,
    max_individual_points: usize,
    genetics: G,
}

impl<G> GeneticEngine<G>
where
    G: Genetics,
{
    pub(crate) fn new(builder: GeneticEngineBuilder<G>) -> Self {
        let rng = match builder.seed {
            Some(seed) => StdRng::seed_from_u64(seed),
            None => StdRng::from_rng(&mut rand::rng()),
        };

        GeneticEngine {
            rng,
            mutation_rate: builder.mutation_rate,
            crossover_rate: builder.crossover_rate,
            max_mutation_points: builder.max_mutation_points,
            max_crossover_points: builder.max_crossover_points,
            max_individual_points: builder.max_individual_points,
            genetics: builder.genetics.unwrap(),
        }
    }

    /// Allows crate access to the random number generator
    pub(crate) fn rng(&mut self) -> &mut StdRng {
        &mut self.rng
    }

    fn random_zero_to_n(&mut self, n: u8) -> u8 {
        self.rng.random::<u8>() % n
    }

    /// Produces a random individual of up to the `max_points` number of code items.
    pub fn rand_individual(&mut self) -> u64 {
        self.genetics
            .random_individual(&mut self.rng, self.max_individual_points)
    }

    /// Produces a random child of the two individuals that is either a mutation of the left individual, or the genetic
    /// crossover of both.
    pub fn rand_child(&mut self, left: u64, right: u64) -> Result<u64, GeneticError> {
        let pick = self.random_zero_to_n(self.mutation_rate + self.crossover_rate);

        if pick < self.mutation_rate {
            let points = self.random_zero_to_n(self.max_mutation_points) + 1;
            Ok(self.genetics.mutate(&mut self.rng, left, points as usize))
        } else {
            let points = self.random_zero_to_n(self.max_crossover_points) + 1;
            Ok(self
                .genetics
                .crossover(&mut self.rng, left, right, points as usize))
        }
    }
}
