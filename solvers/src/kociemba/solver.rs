use std::sync::Arc;

use super::cube::{Cube, Move};
use super::kociemba_tables::KociembaTables;
use super::phase1::Phase1Solver;
use super::phase2::Phase2Solver;
use super::Solver;

pub struct KociembaSolver {
    tables: Arc<KociembaTables>,
}

impl KociembaSolver {
    pub fn new(tables: KociembaTables) -> Self {
        Self {
            tables: Arc::new(tables),
        }
    }
}

impl Solver for KociembaSolver {
    fn solve(&self, mut cube: Cube) -> Vec<Move> {
        let moves1 = Phase1Solver::solve(&cube, &self.tables);
        cube.apply_moves(&moves1);
        let moves2 = Phase2Solver::solve(&cube, &self.tables);

        [moves1, moves2].concat()
    }

    fn name(&self) -> &str {
        "Kociemba's Algorithm"
    }
}
