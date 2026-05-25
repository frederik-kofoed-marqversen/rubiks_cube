use cube::{Cube, Move};

pub trait Solver {
    fn solve(&self, cube: Cube) -> Vec<Move>;
    
    fn name(&self) -> &str {
        "Unknown Solver"
    }
}
