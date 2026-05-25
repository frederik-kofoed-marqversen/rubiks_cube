use std::sync::Arc;

use cube::{Cube, Move, Moveable};
use super::kociemba_tables::KociembaTables;
use super::phase_solvers::{PhaseState, Phase1State, Phase2State, solve_phase, MOVES_PHASE1, MOVES_PHASE2};
use super::Solver;
use std::time::Instant;

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
        let solve_start = Instant::now();
        // Phase 1: Orient all pieces and place E-slice
        let phase1_state = Phase1State::from_cube(&cube, &self.tables);
        let moves1 = solve_phase(phase1_state, &self.tables, 20, &MOVES_PHASE1);
        println!("Phase 1 complete: {} moves", moves1.len());
        // Phase 2: Permute pieces to solved state
        let mut cube_after_phase1 = cube; // Should apply moves to index cube since it is faster. But this can only be done if the state distance is computed after applying moves since the distance should be wrt. phase2 moves, but we are applying phase1 moves.
        cube_after_phase1.apply_moves(&moves1);
        let phase2_state = Phase2State::from_cube(&cube_after_phase1, &self.tables);
        let moves2 = solve_phase(phase2_state, &self.tables, 18, &MOVES_PHASE2);
        println!("Phase 2 complete: {} moves", moves2.len());
        
        let elapsed_time = solve_start.elapsed();
        println!("Solved in {:?}", elapsed_time);
        [moves1, moves2].concat()
    }

    fn name(&self) -> &str {
        "Kociemba's Algorithm"
    }
}
