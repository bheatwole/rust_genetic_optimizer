use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeneticError {
    #[error("Genetics implementation is required")]
    MissingGenetics,

    #[error("Invalid mutation rate: {0}")]
    InvalidMutationRate(u8),

    #[error("Invalid crossover rate: {0}")]
    InvalidCrossoverRate(u8),

    #[error("Invalid number of points: {0}")]
    InvalidPoints(usize),

    #[error("Mutation points must be at least 1 when mutation rate is greater than 0")]
    InvalidMutationPoints,

    #[error("Crossover points must be at least 1 when crossover rate is greater than 0")]
    InvalidCrossoverPoints,

    #[error("individuals_per_island must be greater than 0")]
    InvalidIndividualsPerIsland,

    #[error("elite_individuals_per_generation must be less than individuals_per_island")]
    InvalidEliteCount,

    #[error("max_individual_points must be greater than 0")]
    InvalidIndividualPoints,

    #[error("number_of_individuals_migrating must not exceed individuals_per_island")]
    InvalidMigrationCount,

    #[error("genetic_engine implementation is required")]
    MissingGeneticEngine,
}
