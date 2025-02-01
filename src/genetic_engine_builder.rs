use crate::{GeneticEngine, GeneticError, Genetics};

pub struct GeneticEngineBuilder<G>
where
    G: Genetics,
{
    pub seed: Option<u64>,
    pub mutation_rate: u8,
    pub crossover_rate: u8,
    pub max_mutation_points: u8,
    pub max_crossover_points: u8,
    pub max_individual_points: usize,
    pub genetics: Option<G>,
}

impl<G> Default for GeneticEngineBuilder<G>
where
    G: Genetics,
{
    fn default() -> Self {
        GeneticEngineBuilder {
            seed: None,
            mutation_rate: 1,
            crossover_rate: 9,
            max_mutation_points: 3,
            max_crossover_points: 10,
            max_individual_points: 100,
            genetics: None,
        }
    }
}

impl<G> GeneticEngineBuilder<G>
where
    G: Genetics,
{
    /// Sets the random seed for the genetic engine.
    pub fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Sets the mutation rate. The `mutation_rate` and `crossover_rate` are summed and then a
    /// random value is picked in that range to the final rate is dependant upon both values.
    ///
    /// Set this to zero to disable mutation entirely.
    ///
    /// Default: 1 (approximately 10% when using default crossover_rate)
    pub fn mutation_rate(mut self, rate: u8) -> Self {
        self.mutation_rate = rate;
        self
    }

    /// Sets the crossover rate. The `mutation_rate` and `crossover_rate` are summed and then a
    /// random value is picked in that range to the final rate is dependant upon both values.
    ///
    /// Set this to zero to disable crossover entirely.
    ///
    /// Default: 9 (approximately 90% when using default mutation_rate)
    pub fn crossover_rate(mut self, rate: u8) -> Self {
        self.crossover_rate = rate;
        self
    }

    /// Sets the maximum number of points that will be mutated when the 'Mutation' operation is
    /// chosen. The actual value is random between one and this number. Must be at least one if
    /// mutation is used at all.
    ///
    /// Default: 1
    pub fn max_mutation_points(mut self, points: u8) -> Self {
        self.max_mutation_points = points;
        self
    }

    /// Sets the maximum number of points that will be swapped when the 'Crossover' operation is
    /// chosen. The actual value is random between one and this number. Must be at least one if
    /// crossover is used at all.
    ///
    /// Default: 2
    pub fn max_crossover_points(mut self, points: u8) -> Self {
        self.max_crossover_points = points;
        self
    }

    /// Sets the maximum number of points that an individual can have.
    ///
    /// Default: 100
    pub fn max_individual_points(mut self, points: usize) -> Self {
        self.max_individual_points = points;
        self
    }

    /// Sets the genetics implementation for the genetic engine, which determines how specifically
    /// the individuals are represented and how they are mutated and combined.
    ///
    /// This must be set before calling `build`.
    ///
    /// Default: None
    pub fn genetics(mut self, genetics: G) -> Self {
        self.genetics = Some(genetics);
        self
    }

    /// Consumes the builder and returns a new `GeneticEngine`.
    pub fn build(self) -> Result<GeneticEngine<G>, GeneticError> {
        // A genetics implementation is required.
        if self.genetics.is_none() {
            return Err(GeneticError::MissingGenetics);
        }

        // The max_mutation_points must be at least one if mutation is used at all.
        if self.max_mutation_points < 1 && self.mutation_rate > 0 {
            return Err(GeneticError::InvalidMutationPoints);
        }

        // The max_crossover_points must be at least one if crossover is used at all.
        if self.max_crossover_points < 1 && self.crossover_rate > 0 {
            return Err(GeneticError::InvalidCrossoverPoints);
        }

        // The max_individual_points must be greater than zero
        if self.max_individual_points == 0 {
            return Err(GeneticError::InvalidIndividualPoints);
        }

        Ok(GeneticEngine::new(self))
    }
}
