use super::indexers::Indexer;
use super::phase_states::*;
use super::tables::{compute_min_distance, KociembaTables, PruningTable};
use super::Solver;
use cube::{Cube, Move, Moveable};
use std::time::Instant;

pub trait SearchState: Copy {
    fn is_solved(&self) -> bool;
    fn heuristic(&self) -> u32;
    fn turn(&self, mv: Move) -> Self;
}

#[derive(Copy, Clone)]
pub struct Phase1SearchState<'a> {
    state: Phase1State<'a>,
    distance: u32,
    indexer: Phase1Indexer<'a>,
    prune_table: &'a PruningTable,
}

impl<'a> Phase1SearchState<'a> {
    pub fn new(cube: &Cube, indexer: Phase1Indexer<'a>, tables: &'a KociembaTables) -> Self {
        let state = Phase1State::from_cube(cube, &tables.move_tables);
        let prune_table = &tables.pruning_tables.phase1_prune;
        let distance = compute_min_distance(state, &indexer, prune_table, &MOVES_PHASE1);

        Self {
            state,
            distance,
            indexer,
            prune_table,
        }
    }
}

impl SearchState for Phase1SearchState<'_> {
    fn is_solved(&self) -> bool {
        self.state.is_solved()
    }

    fn heuristic(&self) -> u32 {
        self.distance // Single pruning table
    }

    fn turn(&self, mv: Move) -> Self {
        let idx = self.indexer.to_index(&self.state);
        let mut new_state = *self;
        new_state.state.turn(mv);
        new_state.distance = self.prune_table.get(idx, self.distance);
        new_state
    }
}

#[derive(Copy, Clone)]
pub struct Phase2SearchState<'a> {
    state: Phase2State<'a>,
    distance1: u32,
    distance2: u32,
    indexer1: Phase2Indexer1<'a>,
    indexer2: Phase2Indexer2<'a>,
    pruning_table1: &'a PruningTable,
    pruning_table2: &'a PruningTable,
}

impl<'a> Phase2SearchState<'a> {
    pub fn new(
        cube: &Cube,
        indexer1: Phase2Indexer1<'a>,
        indexer2: Phase2Indexer2<'a>,
        tables: &'a KociembaTables,
    ) -> Self {
        let state = Phase2State::from_cube(cube, &tables.move_tables);
        let pruning_table1 = &tables.pruning_tables.phase2_prune1;
        let pruning_table2 = &tables.pruning_tables.phase2_prune2;
        let distance1 = compute_min_distance(state, &indexer1, pruning_table1, &MOVES_PHASE2);
        let distance2 = compute_min_distance(state, &indexer2, pruning_table2, &MOVES_PHASE2);

        Self {
            state,
            distance1,
            distance2,
            indexer1,
            indexer2,
            pruning_table1,
            pruning_table2,
        }
    }
}

impl SearchState for Phase2SearchState<'_> {
    fn is_solved(&self) -> bool {
        self.state.is_solved()
    }

    fn heuristic(&self) -> u32 {
        // TWO pruning tables - take max!
        self.distance1.max(self.distance2)
    }

    fn turn(&self, mv: Move) -> Self {
        let mut new_state = *self;
        new_state.state.turn(mv);
        let idx1 = self.indexer1.to_index(&new_state.state);
        let idx2 = self.indexer2.to_index(&new_state.state);
        new_state.distance1 = self.pruning_table1.get(idx1, self.distance1);
        new_state.distance2 = self.pruning_table2.get(idx2, self.distance2);
        new_state
    }
}

pub struct KociembaSolver<'a> {
    tables: &'a KociembaTables,
}

impl<'a> KociembaSolver<'a> {
    pub fn new(tables: &'a KociembaTables) -> Self {
        Self { tables }
    }
}

impl Solver for KociembaSolver<'_> {
    fn solve(&self, scrambled_cube: Cube) -> Option<Vec<Move>> {
        let solve_start = Instant::now();
        let mut cube = scrambled_cube;

        // Phase 1: Orient all pieces and place E-slice
        let phase1_indexer =
            Phase1Indexer::new(&self.tables.move_tables, &self.tables.symmetry_tables);
        let state = Phase1SearchState::new(&cube, phase1_indexer, &self.tables);
        let moves1 = ida_star(state, 20, &MOVES_PHASE1, &self.tables);
        println!("Phase 1 complete: {} moves", moves1.len());

        // Apply phase 1 solution to cube to get new state for phase 2
        cube.apply_moves(&moves1);
        assert!(Phase1State::from_cube(&cube, &self.tables.move_tables).is_solved());

        // Phase 2: Permute pieces to solved state
        let phase2_indexer1 =
            Phase2Indexer1::new(&self.tables.move_tables, &self.tables.symmetry_tables);
        let phase2_indexer2 = Phase2Indexer2::new(&self.tables.move_tables);
        let state = Phase2SearchState::new(&cube, phase2_indexer1, phase2_indexer2, &self.tables);
        let moves2 = ida_star(state, 18, &MOVES_PHASE2, &self.tables);
        println!("Phase 2 complete: {} moves", moves2.len());

        // Return final solution
        let elapsed_time = solve_start.elapsed();
        println!("Solved in {:?}", elapsed_time);
        Some([moves1, moves2].concat())
    }

    fn name(&self) -> &str {
        "Kociemba's Algorithm"
    }
}

pub fn ida_star<S: SearchState>(
    start: S,
    max_depth: u32,
    moves: &[Move],
    tables: &KociembaTables,
) -> Vec<Move> {
    if start.is_solved() {
        return Vec::new();
    }

    let mut bound = start.heuristic();
    loop {
        if bound > max_depth {
            panic!("Failed to find a solution within {max_depth} moves");
        }

        let mut path = Vec::new();
        if _ida_star(start, 0, bound, &mut path, moves, tables) {
            return path;
        }

        bound += 1;
    }
}

fn _ida_star<S: SearchState>(
    state: S,
    depth: u32,
    bound: u32,
    path: &mut Vec<Move>,
    moves: &[Move],
    tables: &KociembaTables,
) -> bool {
    let h = state.heuristic();
    if depth + h > bound {
        return false;
    }
    if state.is_solved() {
        return true;
    }

    for &mv in moves {
        if !is_valid_move(path.last(), mv) {
            continue;
        }

        let next_state = state.turn(mv);
        path.push(mv);
        if _ida_star(next_state, depth + 1, bound, path, moves, tables) {
            return true;
        }
        path.pop();
    }

    false
}

fn is_valid_move(prev_opt: Option<&Move>, next: Move) -> bool {
    let Some(prev) = prev_opt else { return true };

    let prev_face = prev.face();
    let next_face = next.face();

    if prev_face == next_face {
        return false;
    }

    // Only allow moves on opposite faces if they are in the correct order
    // E.g. R followed by L is allowed, but L followed by R is not, to avoid redundant sequences like R L R'
    if prev_face.opposite() == next_face {
        return prev_face < next_face;
    }

    true
}
