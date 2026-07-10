use super::indexers::Indexer;
use super::phase_states::*;
use super::tables::{compute_min_distance, KociembaTables, PruningTable};
use super::Solver;
use cube::{Cube, Move, Moveable};
use std::time::{Duration, Instant};

pub trait SearchState: Copy {
    fn is_solved(&self) -> bool;
    fn heuristic(&self) -> u32;
    fn turn(&mut self, mv: Move) -> &mut Self;

    fn apply_moves(&mut self, moves: &[Move]) -> &mut Self {
        for &turn in moves {
            self.turn(turn);
        }
        self
    }
}

#[derive(Copy, Clone)]
pub struct Phase1SearchState<'a> {
    state: Phase1State<'a>,
    distance: u32,
    indexer: Phase1Indexer<'a>,
    prune_table: &'a PruningTable,
}

impl<'a> Phase1SearchState<'a> {
    pub fn new(cube: &Cube, tables: &'a KociembaTables) -> Self {
        let state = Phase1State::from_cube(cube, &tables.move_tables);
        let prune_table = &tables.pruning_tables.phase1_prune;
        let indexer = Phase1Indexer::new(&tables.move_tables, &tables.symmetry_tables);
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

    fn turn(&mut self, mv: Move) -> &mut Self {
        self.state.turn(mv);
        let idx = self.indexer.to_index(&self.state);
        self.distance = self.prune_table.get(idx, self.distance);
        self
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
    pub fn new(cube: &Cube, tables: &'a KociembaTables) -> Self {
        let state = Phase2State::from_cube(cube, &tables.move_tables);
        let pruning_table1 = &tables.pruning_tables.phase2_prune1;
        let pruning_table2 = &tables.pruning_tables.phase2_prune2;
        let indexer1 = Phase2Indexer1::new(&tables.move_tables, &tables.symmetry_tables);
        let indexer2 = Phase2Indexer2::new(&tables.move_tables);
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

    fn turn(&mut self, mv: Move) -> &mut Self {
        self.state.turn(mv);
        let idx1 = self.indexer1.to_index(&self.state);
        let idx2 = self.indexer2.to_index(&self.state);
        self.distance1 = self.pruning_table1.get(idx1, self.distance1);
        self.distance2 = self.pruning_table2.get(idx2, self.distance2);
        self
    }
}

pub struct SearchContext {
    cube: Cube,
    start_time: Instant,
    timeout: Option<Duration>,
    solutions: Vec<Vec<Move>>,
}

impl SearchContext {
    pub fn new(cube: Cube, timeout: Option<Duration>) -> Self {
        Self {
            cube,
            start_time: Instant::now(),
            timeout,
            solutions: Vec::new(),
        }
    }
}

pub struct KociembaSolver<'a> {
    tables: &'a KociembaTables,
}

impl<'a> KociembaSolver<'a> {
    pub fn new(tables: &'a KociembaTables) -> Self {
        Self { tables }
    }

    fn prune(
        heuristic: u32,
        current_length: u32,
        remaining_depth: u32,
        ctx: &SearchContext,
    ) -> bool {
        if let Some(timeout) = ctx.timeout {
            if ctx.start_time.elapsed() > timeout {
                return true;
            }
        }

        let current_best = ctx.solutions.last().map_or(u32::MAX, |s| s.len() as u32); // TODO: Should probably store this number in ctx?
        if heuristic + current_length >= current_best {
            return true;  // Can't beat already found best solution
        }
        if heuristic > remaining_depth {
            return true;  // Can't reach solved state within remaining depth
        }

        false
    }

    fn is_valid_move(prev_opt: Option<Move>, next: Move) -> bool {
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

    fn valid_moves(prev_opt: Option<Move>, moves: &[Move]) -> impl Iterator<Item = Move> + '_ {
        moves
            .iter()
            .copied()
            .filter(move |&mv| Self::is_valid_move(prev_opt, mv))
    }

    fn search_phase2(
        &self,
        state: Phase2SearchState,
        ctx: &mut SearchContext,
        path: &mut Vec<Move>,
        remaining_depth: u32,
    ) {        
        if Self::prune(state.heuristic(), path.len() as u32, remaining_depth, ctx) {
            return;
        }

        if state.is_solved() {
            ctx.solutions.push(path.clone());
            return;
        }

        let prev_mv = path.last().copied();
        let num_solutions = ctx.solutions.len();
        for mv in Self::valid_moves(prev_mv, &MOVES_PHASE2) {
            let mut next_state = state;
            next_state.turn(mv);
            path.push(mv);
            self.search_phase2(next_state, ctx, path, remaining_depth - 1);
            path.pop();

            if ctx.solutions.len() > num_solutions {
                return;
            }
        }
    }

    fn search_phase1(
        &self,
        state: Phase1SearchState,
        ctx: &mut SearchContext,
        path: &mut Vec<Move>,
        remaining_depth: u32,
    ) {
        if Self::prune(state.heuristic(), path.len() as u32, remaining_depth, ctx) {
            return;
        }

        // If phase 1 is solved, transition to phase 2
        if state.is_solved() {
            let mut cube = ctx.cube.clone();
            cube.apply_moves(path); // TODO: Should not use Cube repr. since that is slow!
            let phase2_state = Phase2SearchState::new(&cube, self.tables);

            let num_solutions_before = ctx.solutions.len(); // TODO: Don't like this solution for early exit. Also used in phase 2 search.
            for depth in phase2_state.heuristic()..20 {
                self.search_phase2(phase2_state, ctx, path, depth);
                if ctx.solutions.len() > num_solutions_before {
                    break;
                }
            }
            return;
        }

        let prev_mv = path.last().copied();
        for mv in Self::valid_moves(prev_mv, &MOVES_PHASE1) {
            let mut next_state = state;
            next_state.turn(mv);
            path.push(mv);
            self.search_phase1(next_state, ctx, path, remaining_depth - 1);
            path.pop();
        }
    }

    pub fn run(&self, mut ctx: SearchContext) -> Option<Vec<Move>> {
        ctx.start_time = Instant::now();
        let phase1_state = Phase1SearchState::new(&ctx.cube, self.tables);
        let mut path = Vec::with_capacity(25);
        for depth in phase1_state.heuristic()..=20 {
            self.search_phase1(phase1_state, &mut ctx, &mut path, depth);
            debug_assert!(path.is_empty());
        }

        ctx.solutions.pop()
    }
}

impl Solver for KociembaSolver<'_> {
    fn solve(&self, scrambled_cube: Cube) -> Option<Vec<Move>> {
        let ctx = SearchContext {
            cube: scrambled_cube,
            start_time: Instant::now(),
            timeout: Some(Duration::from_secs(10)),
            solutions: Vec::new(),
        };
        self.run(ctx)
    }

    fn name(&self) -> &str {
        "Kociemba's Algorithm"
    }
}
