pub trait IslandEngine {
    /// Trait implementations can use this callback to configure any data that will apply to all individuals in this
    /// generation. Called once before any individuals are run. The default implementation does nothing.
    fn pre_generation_run(&mut self, _individuals: &[u64]) {}

    /// Trait implementations can use this callback to perform any cleanup for this generation. Called once after all
    /// individuals are run. The default implementation does nothing.
    fn post_generation_run(&mut self, _individuals: &[u64]) {}

    /// Run the virtual machine for a single individual. Called once for each individual on the island.
    ///
    /// A typical implementation might look like the following:
    /// ```ignore
    /// fn run_individual(&mut self, id: u64) {
    ///     // Perform any simulation setup
    ///     let state = MyState::new();
    ///
    ///     // Execute the individual's code. Note that the individual temporarily owns the state.
    ///     let individual = self.get_individual_mut(id);
    ///     let (state, result) = individual.execute(state, (param1, param2));
    ///
    ///     // Calculate how fit this individual is, and store that value. This is the where each island will emphasize
    ///     // a different feature of an individual. One island may place a higher value on code size, another on
    ///     // 'winning' at any cost, another on 'not losing', etc
    ///     individual.set_run_result(Some(my_calculate_fitness_for_island_x(state, result)))
    /// }
    /// ```
    ///
    /// In a simulation where the inputs do not vary from generation to generation, the implementation may wish to check
    /// to see if a RunResult has already been saved for each individual, and skipping the function if already
    /// calculated in a previous run.
    fn run_individual(&mut self, id: u64);

    /// Compare two individuals. The sort order is least fit to most fit. Called multiple times by the sorting algorithm
    /// after all individuals have been run. The default implementation sorts based on the score of the two individuals.
    /// You should implement your own sorting function if the order of individual is based upon multiple criteria or a
    /// simple score is impossible to calculate.
    fn sort_individuals(&self, a: u64, b: u64) -> std::cmp::Ordering {
        self.score_individual(a).cmp(&self.score_individual(b))
    }

    /// Score the effectiveness of one individual. The default implementation returns zero, indicating the worst
    /// fitness possible. You should either implement score_individual or sort_individuals. (You may also implement
    /// both). Use the score if it is easy to boil down the run results to a single number.
    ///
    /// The score is also used by the algorithm to determine the best instruction weights, so it can be useful to write
    /// a score function for use with that algorithm, even if your primary method of choosing individual is by
    /// implementing sort_individuals.
    fn score_individual(&self, _id: u64) -> u64 {
        0
    }
}
