use std::sync::Arc;

use crate::kociemba::coord_cube::CoordinateCube;

use super::cube::{Cube, Move};
use super::kociemba_tables::KociembaTables;
use super::phase_solvers::{Phase1Solver, Phase2Solver};
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
    fn solve(&self, cube: Cube) -> Vec<Move> {
        let cube = CoordinateCube::from_cube(&cube);
        let (moves1, cube) = Phase1Solver::solve(cube, &self.tables);
        let moves2 = Phase2Solver::solve(cube, &self.tables);

        [moves1, moves2].concat()
    }

    fn name(&self) -> &str {
        "Kociemba's Algorithm"
    }
}
