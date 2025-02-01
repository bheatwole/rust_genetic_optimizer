use crate::{
    GeneticEngine, GeneticError, Genetics, Island, IslandEngine, MigrationAlgorithm,
    SelectionCurve, World,
};

#[cfg(any(feature = "multi-threaded", feature = "async"))]
use crate::ThreadingModel;

pub struct WorldBuilder<G>
where
    G: Genetics,
{
    /// The number of individuals on each island. Before running a generation, the island will be filled with the
    /// children of genetic selection if there was a previous generation, or new random individuals if there was no
    /// previous generation.
    ///
    /// Default: 100
    pub individuals_per_island: usize,

    /// The number of individuals whose code will be copied as-is to the next generation. This can help preserve highly
    /// fit code. Set to zero to disable elitism. ref https://en.wikipedia.org/wiki/Genetic_algorithm#Elitism
    ///
    /// Default: 2
    pub elite_individuals_per_generation: usize,

    /// After this many generations across all islands, some of the individual will migrate to new islands. Set to zero
    /// to disable automatic migrations.
    ///
    /// Default: 10
    pub generations_between_migrations: usize,

    /// The number of individuals that will migrate from one island to another.
    ///
    /// Default: 10
    pub number_of_individuals_migrating: usize,

    /// When it is time for a migration, a new island will be selected for the individual according to the specified
    /// algorithm.
    ///
    /// Default: MigrationAlgorithm::Circular
    pub migration_algorithm: MigrationAlgorithm,

    /// If false, individuals selected for migration are removed from their home island. If true, the selected
    /// individuals are cloned and the clone is moved.
    ///
    /// Default: true
    pub clone_migrated_individuals: bool,

    /// The SelectionCurve that will be used when choosing which individual will participate in migration.
    ///
    /// Default: SelectionCurve::PreferenceForFit
    pub select_for_migration: SelectionCurve,

    /// The SelectionCurve that will be used when choosing a fit parent for genetic operations.
    ///
    /// Default: SelectionCurve::PreferenceForFit
    pub select_as_parent: SelectionCurve,

    /// The SelectionCurve used when choosing an elite individual to preserve for the next generation.
    ///
    /// Default: SelectionCurve::StrongPreferenceForFit
    pub select_as_elite: SelectionCurve,

    #[cfg(any(feature = "multi-threaded", feature = "async"))]
    /// Determine how the world runs with regards to multi-threading.
    ///
    /// Default: ThreadingModel::None
    pub threading_model: ThreadingModel,

    /// The genetic engine that will be used to perform genetic operations.
    pub genetic_engine: Option<GeneticEngine<G>>,

    /// The islands that exist in the world. At least one is required.
    pub islands: Vec<Island>,
}

impl<G> Default for WorldBuilder<G>
where
    G: Genetics,
{
    fn default() -> Self {
        WorldBuilder {
            individuals_per_island: 100,
            elite_individuals_per_generation: 2,
            generations_between_migrations: 10,
            number_of_individuals_migrating: 10,
            migration_algorithm: MigrationAlgorithm::Circular,
            clone_migrated_individuals: true,
            select_for_migration: SelectionCurve::PreferenceForFit,
            select_as_parent: SelectionCurve::PreferenceForFit,
            select_as_elite: SelectionCurve::StrongPreferenceForFit,
            #[cfg(any(feature = "multi-threaded", feature = "async"))]
            threading_model: ThreadingModel::None,
            genetic_engine: None,
            islands: vec![],
        }
    }
}

impl<G> WorldBuilder<G>
where
    G: Genetics,
{
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_individuals_per_island(mut self, count: usize) -> Self {
        self.individuals_per_island = count;
        self
    }

    pub fn with_elite_individuals(mut self, count: usize) -> Self {
        self.elite_individuals_per_generation = count;
        self
    }

    pub fn with_generations_between_migrations(mut self, generations: usize) -> Self {
        self.generations_between_migrations = generations;
        self
    }

    pub fn with_migrating_individuals(mut self, count: usize) -> Self {
        self.number_of_individuals_migrating = count;
        self
    }

    pub fn with_migration_algorithm(mut self, algorithm: MigrationAlgorithm) -> Self {
        self.migration_algorithm = algorithm;
        self
    }

    pub fn with_clone_migrated_individuals(mut self, clone: bool) -> Self {
        self.clone_migrated_individuals = clone;
        self
    }

    pub fn with_select_for_migration(mut self, curve: SelectionCurve) -> Self {
        self.select_for_migration = curve;
        self
    }

    pub fn with_select_as_parent(mut self, curve: SelectionCurve) -> Self {
        self.select_as_parent = curve;
        self
    }

    pub fn with_select_as_elite(mut self, curve: SelectionCurve) -> Self {
        self.select_as_elite = curve;
        self
    }

    #[cfg(any(feature = "multi-threaded", feature = "async"))]
    pub fn with_threading_model(mut self, model: ThreadingModel) -> Self {
        self.threading_model = model;
        self
    }

    pub fn with_genetic_engine(mut self, engine: GeneticEngine<G>) -> Self {
        self.genetic_engine = Some(engine);
        self
    }

    pub fn add_island<S: Into<String>>(
        &mut self,
        name: S,
        engine: Box<dyn IslandEngine>,
    ) -> &mut Self {
        self.islands.push(Island::new(name, engine));
        self
    }

    pub fn build(self) -> Result<World<G>, GeneticError> {
        // Validate configuration
        if self.individuals_per_island == 0 {
            return Err(GeneticError::InvalidIndividualsPerIsland);
        }

        if self.elite_individuals_per_generation >= self.individuals_per_island {
            return Err(GeneticError::InvalidEliteCount);
        }

        if self.number_of_individuals_migrating > self.individuals_per_island {
            return Err(GeneticError::InvalidMigrationCount);
        }

        if self.genetic_engine.is_none() {
            return Err(GeneticError::MissingGeneticEngine);
        }

        Ok(World::new(self))
    }
}
