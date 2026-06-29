use cube::{Cube, Move};

pub trait Solver {
    fn solve(&self, cube: Cube) -> Option<Vec<Move>>;
    
    fn name(&self) -> &str {
        "Unknown Solver"
    }
}
