/// Defines the method by which individuals migrate from island to island when it is time for a migration.
#[derive(Clone, Debug, PartialEq)]
pub enum MigrationAlgorithm {
    /// The islands are arranged in a circle and individuals always migrate one island clockwise.
    Circular,

    /// The islands are arranged in a circle and individuals migrate to the island 'n' number of islands clockwise. 'n'
    /// must be in the range 1..number of islands and will be taken 'mod number of islands' if it is larger.
    /// MigrationAlgorithm::Circular is equivalent to MigrationAlgorithm::Cyclical(1)
    Cyclical(usize),

    /// The islands are arranged in a circle and individuals migrate to the island 'n' number of islands clockwise in
    /// the same manner as `Cyclical(n)`. However, Incremental increases 'n' after every migration, wrapping around back
    /// to 1 when 'n' == number of islands.
    Incremental(usize),

    /// Individuals migrate as for `Circular` but the order of the circle is randomized after every migration.
    RandomCircular,

    /// Every individual selected for migration picks a completely random island that is not its current island and
    /// migrates to that island.
    CompletelyRandom,
}
