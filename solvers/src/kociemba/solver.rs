use std::sync::Arc;

use super::cube::{Cube, Move};
use super::kociemba_tables::KociembaTables;
use super::phase_solvers::{Phase1State, Phase2State, solve_phase};
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
        // Phase 1: Orient all pieces and place E-slice
        let phase1_state = Phase1State::from_cube(&cube);
        let moves1 = solve_phase(phase1_state, &self.tables, 20);
        
        // Phase 2: Permute pieces to solved state
        let mut phase2_state = Phase2State::from_cube(&cube);
        phase2_state = phase2_state.apply_moves(&moves1, &self.tables);
        let moves2 = solve_phase(phase2_state, &self.tables, 18);
        
        [moves1, moves2].concat()
    }

    fn name(&self) -> &str {
        "Kociemba's Algorithm"
    }
}
