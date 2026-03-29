use crate::cube::{Cube, Move};

/// Generic trait for Rubik's Cube solvers
/// 
/// # Usage Patterns for Table-based Solvers
/// 
/// ```rust,ignore
/// // Pattern 1: Automatic (recommended) - load or build
/// let solver = ThistlethwaiteSolver::new()
///     .with_tables("./data")
///     .expect("Failed to initialize");
/// 
/// // Pattern 2: Explicit control
/// let mut solver = ThistlethwaiteSolver::new();
/// if solver.load_tables("./data").is_err() {
///     println!("Building tables from scratch...");
///     solver.build_tables()?;
///     solver.save_tables("./data")?;
/// }
/// 
/// // Pattern 3: Always build fresh (useful for testing)
/// let mut solver = ThistlethwaiteSolver::new();
/// solver.build_tables()?;
/// 
/// // Pattern 4: Build once, save for later
/// let mut solver = ThistlethwaiteSolver::new();
/// solver.build_tables()?;
/// solver.save_tables("./my_custom_path")?;
/// ```
pub trait Solver: Sized {
    /// Solve a scrambled cube, returning a sequence of moves
    fn solve(&self, cube: &Cube) -> Vec<Move>;
    
    /// Build lookup tables from scratch (optional, for table-based solvers)
    /// Default implementation is a no-op for solvers that don't need tables
    fn build_tables(&mut self) -> Result<(), std::io::Error> {
        Ok(())
    }
    
    /// Load precomputed tables from directory (optional)
    /// Default implementation is a no-op for solvers that don't need tables
    fn load_tables(&mut self, _data_dir: &str) -> Result<(), std::io::Error> {
        Ok(())
    }
    
    /// Save tables to directory (optional)
    /// Default implementation is a no-op for solvers that don't need tables
    fn save_tables(&self, _data_dir: &str) -> Result<(), std::io::Error> {
        Ok(())
    }
    
    /// Check if solver is ready to use (has tables loaded if needed)
    /// Default implementation returns true (always ready)
    fn is_ready(&self) -> bool {
        true
    }
    
    /// Builder pattern: Initialize solver with tables from directory
    /// Attempts to load tables from disk, builds from scratch if not found
    fn with_tables(mut self, data_dir: &str) -> Result<Self, std::io::Error> {
        match self.load_tables(data_dir) {
            Ok(()) => {
                if !self.is_ready() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Tables loaded but solver not ready"
                    ));
                }
                println!("Loaded tables from {}", data_dir);
                Ok(self)
            }
            Err(_) => {
                println!("Tables not found in {}, building from scratch...", data_dir);
                self.build_tables()?;
                self.save_tables(data_dir)?;
                println!("Saved tables to {}", data_dir);
                Ok(self)
            }
        }
    }
    
    /// Return solver name for debugging/benchmarking
    fn name(&self) -> &str {
        "Unknown Solver"
    }
}
