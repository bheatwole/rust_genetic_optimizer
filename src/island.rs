use crate::{IslandEngine, SelectionCurve};

pub struct Island {
    name: String,
    engine: Box<dyn IslandEngine>,
    individuals: Vec<u64>,
    individuals_are_sorted: bool,
    future: Vec<u64>,
}

impl Island {
    pub(crate) fn new<S: Into<String>>(name: S, engine: Box<dyn IslandEngine>) -> Island {
        Island {
            name: name.into(),
            engine,
            individuals: vec![],
            individuals_are_sorted: false,
            future: vec![],
        }
    }

    /// Returns the name of the island
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Resets the island to it's 'new' state.
    pub fn clear(&mut self) {
        self.individuals.clear();
        self.individuals_are_sorted = false;
        self.future.clear();
    }

    /// Returns the most fit of all the individuals (the one sorted to the tail by the sorting algorithm). Returns None
    /// if there are no Individuals or if the individuals have not been sorted
    pub fn most_fit_individual(&self) -> Option<u64> {
        if !self.individuals_are_sorted {
            return None;
        }
        self.individuals.last().map(|x| *x)
    }

    /// Returns the least fit of all the individuals (the one sorted to the head by the sorting algorithm). Returns None
    /// if there are no Individuals or if the individuals have not been sorted
    pub fn least_fit_individual(&self) -> Option<u64> {
        if !self.individuals_are_sorted {
            return None;
        }
        self.individuals.first().map(|x| *x)
    }

    /// Returns one individual by index, or None if the index is out of range
    pub fn get_one_individual(&self, index: usize) -> Option<u64> {
        self.individuals.get(index).map(|x| *x)
    }

    /// Uses the specified VM to run one generation of individuals. Calls all of the user-supplied functions from the
    /// `Island` trait.
    #[cfg(not(feature = "async"))]
    pub fn run_one_generation(&mut self) {
        // Allow the island to set up for all runs
        self.engine.pre_generation_run(&self.individuals);

        // Run each individual
        for &id in &self.individuals[..] {
            self.engine.run_individual(id);
        }

        // Allow the island to before any cleanup or group analysis tasks
        self.engine.post_generation_run(&self.individuals);

        // Sort the individuals
        self.sort_individuals();
    }

    /// Uses the specified VM to run one generation of individuals. Calls all of the user-supplied functions from the
    /// `Island` trait.
    #[cfg(feature = "async")]
    pub async fn run_one_generation(&mut self) {
        // Allow the island to set up for all runs
        self.engine.pre_generation_run(&self.individuals).await;

        // Run each individual
        for &id in &self.individuals[..] {
            self.engine.run_individual(id);
        }

        // Allow the island to before any cleanup or group analysis tasks
        self.engine.post_generation_run(&self.individuals).await;

        // Sort the individuals
        self.sort_individuals();
    }

    /// Sorts the individuals by calling the sorter function.
    pub fn sort_individuals(&mut self) {
        self.individuals
            .sort_by(|a, b| self.engine.sort_individuals(*a, *b));
        self.individuals_are_sorted = true;
    }

    /// Returns the current number of individuals on the island.
    pub fn len(&self) -> usize {
        self.individuals.len()
    }

    /// Returns the number of individuals in the next generation
    pub fn len_future_generation(&self) -> usize {
        self.future.len()
    }

    /// Permanently removes all of the current generation and sets the future generation as the current generation.
    pub fn advance_generation(&mut self) {
        self.individuals.clear();
        self.individuals_are_sorted = false;
        std::mem::swap(&mut self.individuals, &mut self.future);
    }

    /// Select one individual from the island according to the specified SelectionCurve and borrow it.
    /// Returns the individual borrowed or None if the population is zero or not sorted
    pub fn select_one_individual<Rnd: rand::Rng>(
        &self,
        curve: SelectionCurve,
        rng: &mut Rnd,
    ) -> Option<u64> {
        if !self.individuals_are_sorted {
            return None;
        }

        let max = self.individuals.len();
        if max == 0 {
            None
        } else {
            self.individuals
                .get(curve.pick_one_index(rng, max))
                .map(|x| *x)
        }
    }

    /// Select one individual from the island according to the specified SelectionCurve and remove it permanently.
    /// Returns the individual removed or None if the population is zero or not sorted
    pub fn select_and_remove_one_individual<Rnd: rand::Rng>(
        &mut self,
        curve: SelectionCurve,
        rng: &mut Rnd,
    ) -> Option<u64> {
        if !self.individuals_are_sorted {
            return None;
        }

        let max = self.individuals.len();
        if max == 0 {
            None
        } else {
            Some(self.individuals.remove(curve.pick_one_index(rng, max)))
        }
    }

    /// Adds an individual to the future generation
    pub fn add_individual_to_future_generation(&mut self, id: u64) {
        self.future.push(id);
    }

    /// Returns the score for the individual specified by index, or None if the index is out of bounds
    pub fn score_for_individual(&self, index: usize) -> Option<u64> {
        if let Some(individual) = self.get_one_individual(index) {
            Some(self.engine.score_individual(individual))
        } else {
            None
        }
    }
}
