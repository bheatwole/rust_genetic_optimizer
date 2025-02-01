use rand::seq::SliceRandom;
use rand::Rng;

#[cfg(any(feature = "multi-threaded", feature = "async"))]
use crate::ThreadingModel;
use crate::*;

pub struct World<G>
where
    G: Genetics,
{
    // Configuration
    individuals_per_island: usize,
    elite_individuals_per_generation: usize,
    generations_between_migrations: usize,
    number_of_individuals_migrating: usize,
    migration_algorithm: MigrationAlgorithm,
    clone_migrated_individuals: bool,
    select_for_migration: SelectionCurve,
    select_as_parent: SelectionCurve,
    select_as_elite: SelectionCurve,
    #[cfg(any(feature = "multi-threaded", feature = "async"))]
    threading_model: ThreadingModel,
    genetic_engine: GeneticEngine<G>,

    // Runtime state
    islands: Vec<Island>,
    generation_count: usize,
    generations_remaining_before_migration: usize,
}

impl<G> World<G>
where
    G: Genetics,
{
    pub(crate) fn new(builder: WorldBuilder<G>) -> Self {
        World {
            individuals_per_island: builder.individuals_per_island,
            elite_individuals_per_generation: builder.elite_individuals_per_generation,
            generations_between_migrations: builder.generations_between_migrations,
            number_of_individuals_migrating: builder.number_of_individuals_migrating,
            migration_algorithm: builder.migration_algorithm,
            clone_migrated_individuals: builder.clone_migrated_individuals,
            select_for_migration: builder.select_for_migration,
            select_as_parent: builder.select_as_parent,
            select_as_elite: builder.select_as_elite,
            #[cfg(any(feature = "multi-threaded", feature = "async"))]
            threading_model: builder.threading_model,
            genetic_engine: builder.genetic_engine.unwrap(),
            islands: builder.islands,
            generation_count: 0,
            generations_remaining_before_migration: builder.generations_between_migrations,
        }
    }

    /// Returns the total number of islands
    pub fn get_number_of_islands(&self) -> usize {
        self.islands.len()
    }

    /// Borrows an island by the specified index
    pub fn get_island(&self, index: usize) -> Option<&Island> {
        self.islands.get(index)
    }

    /// Mutably borrows an island by the specified index
    pub fn get_island_mut(&mut self, index: usize) -> Option<&mut Island> {
        self.islands.get_mut(index)
    }

    /// Borrows an island by the specified name
    pub fn get_island_by_name(&self, name: &str) -> Option<&Island> {
        self.islands.iter().find(|island| island.name() == name)
    }

    /// Removes all individuals from all islands
    pub fn reset_all_islands(&mut self) {
        for island in self.islands.iter_mut() {
            island.clear();
        }
    }

    /// Runs the next generation across all islands.
    #[cfg(not(feature = "async"))]
    pub fn run_one_generation(&mut self) {
        for island in self.islands.iter_mut() {
            island.run_one_generation();
        }

        // See if it is time for a migration
        if self.generations_between_migrations > 0 {
            self.generations_remaining_before_migration -= 1;
            if self.generations_remaining_before_migration == 0 {
                self.migrate_individuals_between_islands();
                self.generations_remaining_before_migration = self.generations_between_migrations;
            }
        }
    }

    /// Runs the next generation across all islands.
    #[cfg(feature = "async")]
    pub async fn run_one_generation(&mut self) {
        for island in self.islands.iter_mut() {
            island.run_one_generation().await;
        }

        // See if it is time for a migration
        if self.generations_between_migrations > 0 {
            self.generations_remaining_before_migration -= 1;
            if self.generations_remaining_before_migration == 0 {
                self.migrate_individuals_between_islands();
                self.generations_remaining_before_migration = self.generations_between_migrations;
            }
        }
    }

    /// Fills all islands with the children of the genetic algorithm, or with random individuals if there was no
    /// previous generation from which to draw upon.
    pub fn fill_all_islands(&mut self) -> Result<(), GeneticError> {
        for id in 0..self.islands.len() {
            let mut elite_remaining = self.elite_individuals_per_generation;
            while self.len_island_future_generation(id) < self.individuals_per_island {
                let island = self.islands.get(id).unwrap();
                let pick_elite = if elite_remaining > 0 {
                    elite_remaining -= 1;
                    true
                } else {
                    false
                };
                let next = if island.len() == 0 {
                    self.genetic_engine.rand_individual()
                } else {
                    if pick_elite {
                        let elite = island
                            .select_one_individual(self.select_as_elite, self.genetic_engine.rng())
                            .unwrap();

                        elite.clone()
                    } else {
                        let left = island
                            .select_one_individual(self.select_as_parent, self.genetic_engine.rng())
                            .unwrap();
                        let right = island
                            .select_one_individual(self.select_as_parent, self.genetic_engine.rng())
                            .unwrap();
                        self.genetic_engine.rand_child(left, right)?
                    }
                };
                self.add_individual_to_island_future_generation(id, next);
            }

            // Now that the future generation is full, make it the current generation
            self.advance_island_generation(id);
        }

        Ok(())
    }

    fn len_island_future_generation(&self, index: usize) -> usize {
        self.islands.get(index).unwrap().len_future_generation()
    }

    fn add_individual_to_island_future_generation(&mut self, index: usize, id: u64) {
        self.islands
            .get_mut(index)
            .unwrap()
            .add_individual_to_future_generation(id)
    }

    fn advance_island_generation(&mut self, index: usize) {
        self.islands.get_mut(index).unwrap().advance_generation()
    }

    /// Runs generations until the specified function returns false
    #[cfg(not(feature = "async"))]
    pub fn run_generations_while<While>(&mut self, mut while_fn: While) -> Result<(), GeneticError>
    where
        While: FnMut(&World<G>) -> bool,
    {
        // Always run at least one generation
        let mut running = true;
        while running {
            self.fill_all_islands()?;
            self.run_one_generation();
            running = while_fn(self);
        }

        Ok(())
    }

    /// Runs generations until the specified function returns false
    #[cfg(feature = "async")]
    pub async fn run_generations_while<While>(
        &mut self,
        mut while_fn: While,
    ) -> Result<(), GeneticError>
    where
        While: FnMut(&World<G>) -> bool,
    {
        // Always run at least one generation
        let mut running = true;
        while running {
            self.fill_all_islands()?;
            self.run_one_generation().await;
            running = while_fn(self);
        }

        Ok(())
    }

    pub fn migrate_individuals_between_islands(&mut self) {
        let island_len = self.islands.len();

        // It only makes sense to migrate if there are at least two islands
        if island_len > 1 {
            match self.migration_algorithm {
                MigrationAlgorithm::Circular => self.migrate_all_islands_circular_n(1),
                MigrationAlgorithm::Cyclical(n) => self.migrate_all_islands_circular_n(n),
                MigrationAlgorithm::Incremental(n) => {
                    self.migrate_all_islands_circular_n(n);

                    // Increment 'n'. An 'n' of zero makes no sense, so when it gets there use '1' instead.
                    let mut next_n = self.island_at_distance(0, n + 1);
                    if next_n == 0 {
                        next_n = 1
                    }
                    self.migration_algorithm = MigrationAlgorithm::Incremental(next_n);
                }
                MigrationAlgorithm::RandomCircular => {
                    // Define a new order of islands and calculate the distance to the next island in this new order.
                    // For example, if there are 7 islands and the order starts with 2, 3: the first distance is 1.
                    // However if the order starts with 3, 2: the first distance is 6
                    //
                    // This algorithm achieves the desired goal of having individuals from each island migrate together
                    // to another random island, and each island is the source and destination exactly once.
                    let island_order = self.random_island_order();
                    let distances = World::<G>::distances_to_next_island(&island_order[..]);
                    for (source_id, n) in std::iter::zip(island_order, distances) {
                        self.migrate_one_island_circular_n(source_id, n);
                    }
                }
                MigrationAlgorithm::CompletelyRandom => {
                    let len = self.islands.len();

                    // For each migrating individual on each island, pick a random destination that is not the same
                    // island and migrate there.
                    for source_island_id in 0..len {
                        for _ in 0..self.number_of_individuals_migrating {
                            let mut destination_island_id = source_island_id;
                            while source_island_id != destination_island_id {
                                destination_island_id =
                                    self.genetic_engine.rng().random_range(0..len);
                            }
                            self.migrate_one_individual_from_island_to_island(
                                source_island_id,
                                destination_island_id,
                            );
                        }
                    }
                }
            }
        }
    }

    fn migrate_one_individual_from_island_to_island(
        &mut self,
        source_island_id: usize,
        destination_island_id: usize,
    ) {
        let curve = self.select_for_migration;

        // Get the migrating individual from the source island
        let source_island = self.islands.get_mut(source_island_id).unwrap();
        let migrating: u64 = if self.clone_migrated_individuals {
            source_island
                .select_one_individual(curve, self.genetic_engine.rng())
                .unwrap()
                .clone()
        } else {
            source_island
                .select_and_remove_one_individual(curve, self.genetic_engine.rng())
                .unwrap()
        };

        // Add it to the destination island
        let destination_island = self.islands.get_mut(destination_island_id).unwrap();
        destination_island.add_individual_to_future_generation(migrating);
    }

    // Calculates the ID of the island at a specific distance from the source. Wraps around when we get to the end of
    // the list.
    fn island_at_distance(&self, source_id: usize, distance: usize) -> usize {
        (source_id + distance) % self.islands.len()
    }

    fn migrate_all_islands_circular_n(&mut self, n: usize) {
        for source_island_id in 0..self.islands.len() {
            self.migrate_one_island_circular_n(source_island_id, n);
        }
    }

    fn migrate_one_island_circular_n(&mut self, source_island_id: usize, n: usize) {
        let destination_island_id = self.island_at_distance(source_island_id, n);
        for _ in 0..self.number_of_individuals_migrating {
            self.migrate_one_individual_from_island_to_island(
                source_island_id,
                destination_island_id,
            );
        }
    }

    // Creates a Vec containing the source_id of each island exactly one time
    fn random_island_order(&mut self) -> Vec<usize> {
        let mut island_ids: Vec<usize> = (0..self.islands.len()).collect();
        island_ids.shuffle(self.genetic_engine.rng());

        island_ids
    }

    // Creates a Vec containing the distance to the previous island in the list for every entry in the parameter. The
    // distance for the first entry wraps around to the last item.
    fn distances_to_next_island(island_id: &[usize]) -> Vec<usize> {
        let len = island_id.len();
        let mut distances = Vec::with_capacity(len);
        let mut previous_source_id = island_id.last().unwrap();
        for source_id in island_id.iter() {
            let distance = ((previous_source_id + len) - source_id) % len;
            distances.push(distance);
            previous_source_id = source_id;
        }

        distances
    }

    pub fn generation_count(&self) -> usize {
        self.generation_count
    }
}
