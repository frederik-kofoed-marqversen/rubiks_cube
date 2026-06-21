use super::indexers::Indexer;
use super::tables::{KociembaTables, PruningTable, compute_min_distance};
use cube::{Cube, Move, Moveable};
use super::phase_states::*;
use super::Solver;
use std::time::Instant;

pub trait SearchState: Copy {
    fn is_solved(&self) -> bool;
    fn heuristic(&self) -> u32;
    fn turn(& self, mv: Move) -> Self;
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
        self.distance  // Single pruning table
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
    indexer2: Phase2Indexer2,
    pruning_table1: &'a PruningTable,
    pruning_table2: &'a PruningTable,
}

impl<'a> Phase2SearchState<'a> {
    pub fn new(cube: &Cube, indexer1: Phase2Indexer1<'a>, indexer2: Phase2Indexer2, tables: &'a KociembaTables) -> Self {
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
        let idx1 = self.indexer1.to_index(&self.state);
        let idx2 = self.indexer2.to_index(&self.state);
        let mut new_state = *self;
        new_state.state.turn(mv);
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
    fn solve(&self, scrambled_cube: Cube) -> Vec<Move> {
        let solve_start = Instant::now();
        let mut cube = scrambled_cube;

        // Phase 1: Orient all pieces and place E-slice
        let phase1_indexer = Phase1Indexer{
            eos_reduction_table: &self.tables.symmetry_tables.eos_reduction,
            co_symmetry_table: &self.tables.symmetry_tables.co_conjugation,
        };
        let state = Phase1SearchState::new(&cube, phase1_indexer, &self.tables);
        let moves1 = ida_star(state, 20, &MOVES_PHASE1, &self.tables);
        println!("Phase 1 complete: {} moves", moves1.len());

        // Apply phase 1 solution to cube to get new state for phase 2
        cube.apply_moves(&moves1);

        // Phase 2: Permute pieces to solved state
        let phase2_indexer1 = Phase2Indexer1 {
            cp_reduction_table: &self.tables.symmetry_tables.cp_reduction,
            ud_symmetry_table: &self.tables.symmetry_tables.ud_conjugation,
        };
        let phase2_indexer2 = Phase2Indexer2;
        let state = Phase2SearchState::new(&cube, phase2_indexer1, phase2_indexer2, &self.tables);
        let moves2 = ida_star(state, 18, &MOVES_PHASE2, &self.tables);
        println!("Phase 2 complete: {} moves", moves2.len());

        // Return final solution
        let elapsed_time = solve_start.elapsed();
        println!("Solved in {:?}", elapsed_time);
        [moves1, moves2].concat()
    }

    fn name(&self) -> &str {
        "Kociemba's Algorithm"
    }
}

pub fn ida_star<S: SearchState>(start: S, max_depth: u32, moves: &[Move], tables: &KociembaTables) -> Vec<Move> {
    if start.is_solved() {
        return Vec::new();
    }

    let mut bound = start.heuristic();
    println!("Initial heuristic bound: {}", bound);
    loop {
        if bound > max_depth {
            panic!("Failed to find a solution within {max_depth} moves");
        }
        println!("Searching with bound {}...", bound);

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

#[cfg(test)]
mod tests {
    use super::*;
    use cube::Moveable;
    use std::sync::LazyLock;

    static TABLES: LazyLock<KociembaTables> = LazyLock::new(|| {
        KociembaTables::load_or_build(KociembaTables::DEFAULT_PATH)
            .expect("Failed to load or build tables")
    });

    #[test]
    fn phase1_search_state_heuristic() {
        let cube = Cube::new_solved();
        let indexer = Phase1Indexer {
            eos_reduction_table: &TABLES.symmetry_tables.eos_reduction,
            co_symmetry_table: &TABLES.symmetry_tables.co_conjugation,
        };
        
        let state = Phase1SearchState::new(&cube, indexer, &TABLES);
        assert_eq!(state.heuristic(), 0, "Solved cube should have heuristic 0");
        assert!(state.is_solved());

        // Apply a move and check heuristic increases
        let state = state.turn(Move::R);
        assert!(state.heuristic() > 0, "Scrambled cube should have heuristic > 0");
        assert!(!state.is_solved(), "Scrambled cube should not be solved");
    }
    
    #[test]
    fn phase1_short_scramble() {
        let scramble = vec![Move::R, Move::U, Move::Rp, Move::Up];
        let mut cube = Cube::new_solved();
        cube.apply_moves(&scramble);
        
        let indexer = Phase1Indexer {
            eos_reduction_table: &TABLES.symmetry_tables.eos_reduction,
            co_symmetry_table: &TABLES.symmetry_tables.co_conjugation,
        };
        
        let state = Phase1SearchState::new(&cube, indexer, &TABLES);
        let solution = ida_star(state, 20, &MOVES_PHASE1, &TABLES);
        
        assert!(solution.len() <= 15, "Short scramble should solve efficiently");
        
        // Verify solution
        cube.apply_moves(&solution);
        let final_state = Phase1State::from_cube(&cube, &TABLES.move_tables);
        assert!(final_state.is_solved(), "Solution should reach Phase 1 goal");
    }
    
    #[test]
    fn phase2_solved_state() {
        let cube = Cube::new_solved();
        let indexer1 = Phase2Indexer1 {
            cp_reduction_table: &TABLES.symmetry_tables.cp_reduction,
            ud_symmetry_table: &TABLES.symmetry_tables.ud_conjugation,
        };
        let indexer2 = Phase2Indexer2;
        
        let state = Phase2SearchState::new(&cube, indexer1, indexer2, &TABLES);
        assert_eq!(state.heuristic(), 0, "Solved cube should have heuristic 0");
        assert!(state.is_solved());

        let state = state.turn(Move::R2);
        assert!(state.heuristic() > 0, "Scrambled cube should have heuristic > 0");
        assert!(!state.is_solved(), "Scrambled cube should not be solved");
    }
    
    #[test]
    fn phase2_simple_scramble() {
        let scramble = vec![Move::R2, Move::U2, Move::D, Move::R2];
        let mut cube = Cube::new_solved();
        cube.apply_moves(&scramble);
        
        let indexer1 = Phase2Indexer1 {
            cp_reduction_table: &TABLES.symmetry_tables.cp_reduction,
            ud_symmetry_table: &TABLES.symmetry_tables.ud_conjugation,
        };
        let indexer2 = Phase2Indexer2;
        
        let state = Phase2SearchState::new(&cube, indexer1, indexer2, &TABLES);
        let solution = ida_star(state, 18, &MOVES_PHASE2, &TABLES);
        
        // Verify solution
        cube.apply_moves(&solution);
        let final_state = Phase2State::from_cube(&cube, &TABLES.move_tables);
        assert!(final_state.is_solved(), "Phase 2 should reach goal state");
        assert!(cube.is_solved(), "Final cube state should be solved");
    }
    
    #[test]
    fn full_solve_simple_scramble() {
        // End-to-end test using a simpler direct solve approach
        let scramble = vec![Move::R, Move::U, Move::Rp, Move::Up];
        let mut cube = Cube::new_solved();
        cube.apply_moves(&scramble);
        
        let solver = KociembaSolver::new(&TABLES);
        let solution = solver.solve(cube);
        
        cube.apply_moves(&solution);
        assert!(cube.is_solved(), "Full two-phase solution should solve the cube");
    }
}

// def search_phase2(self, corners, ud_edges, slice_sorted, dist, togo_phase2):
//         # ##############################################################################################################
//         if self.terminated.is_set() or self.phase2_done:
//             return
//         ################################################################################################################
//         if togo_phase2 == 0 and slice_sorted == 0:
//             self.lock.acquire()  # phase 2 solved, store solution
//             man = self.sofar_phase1 + self.sofar_phase2
//             if len(self.solutions) == 0 or (len(self.solutions[-1]) > len(man)):

//                 if self.inv == 1:  # we solved the inverse cube
//                     man = list(reversed(man))
//                     man[:] = [Move((m // 3) * 3 + (2 - m % 3)) for m in man]  # R1->R3, R2->R2, R3->R1 etc.
//                 man[:] = [Move(sy.conj_move[N_MOVE * 16 * self.rot + m]) for m in man]
//                 self.solutions.append(man)
//                 self.shortest_length[0] = len(man)

//             if self.shortest_length[0] <= self.ret_length:  # we have reached the target length
//                 self.terminated.set()
//             self.lock.release()
//             self.phase2_done = True
//         else:
//             for m in Move:
//                 if m in [Move.R1, Move.R3, Move.F1, Move.F3,
//                          Move.L1, Move.L3, Move.B1, Move.B3]:
//                     continue

//                 if len(self.sofar_phase2) > 0:
//                     diff = self.sofar_phase2[-1] // 3 - m // 3
//                     if diff in [0, 3]:  # successive moves: on same face or on same axis with wrong order
//                         continue
//                 else:
//                     if len(self.sofar_phase1) > 0:
//                         diff = self.sofar_phase1[-1] // 3 - m // 3
//                         if diff in [0, 3]:  # successive moves: on same face or on same axis with wrong order
//                             continue

//                 corners_new = mv.corners_move[18 * corners + m]
//                 ud_edges_new = mv.ud_edges_move[18 * ud_edges + m]
//                 slice_sorted_new = mv.slice_sorted_move[18 * slice_sorted + m]

//                 classidx = sy.corner_classidx[corners_new]
//                 sym = sy.corner_sym[corners_new]
//                 dist_new_mod3 = pr.get_corners_ud_edges_depth3(
//                     40320 * classidx + sy.ud_edges_conj[(ud_edges_new << 4) + sym])
//                 dist_new = pr.distance[3 * dist + dist_new_mod3]
//                 if max(dist_new, pr.cornslice_depth[24 * corners_new + slice_sorted_new]) >= togo_phase2:
//                     continue  # impossible to reach solved cube in togo_phase2 - 1 moves

//                 self.sofar_phase2.append(m)
//                 self.search_phase2(corners_new, ud_edges_new, slice_sorted_new, dist_new, togo_phase2 - 1)
//                 self.sofar_phase2.pop(-1)

//     def search(self, flip, twist, slice_sorted, dist, togo_phase1):
//         # ##############################################################################################################
//         if self.terminated.is_set():
//             return
//         ################################################################################################################
//         if togo_phase1 == 0:  # phase 1 solved

//             if time.monotonic() > self.start_time + self.timeout and len(self.solutions) > 0:
//                 self.terminated.set()

//             # compute initial phase 2 coordinates
//             if self.sofar_phase1:  # check if list is not empty
//                 m = self.sofar_phase1[-1]
//             else:
//                 m = Move.U1  # value is irrelevant here, no phase 1 moves

//             if m in [Move.R3, Move.F3, Move.L3, Move.B3]:  # phase 1 solution come in pairs
//                 corners = mv.corners_move[18 * self.cornersave + m - 1]  # apply R2, F2, L2 ord B2 on last ph1 solution
//             else:
//                 corners = self.co_cube.corners
//                 for m in self.sofar_phase1:  # get current corner configuration
//                     corners = mv.corners_move[18 * corners + m]
//                 self.cornersave = corners

//             # new solution must be shorter and we do not use phase 2 maneuvers with length > 11 - 1 = 10
//             togo2_limit = min(self.shortest_length[0] - len(self.sofar_phase1), 11)
//             if pr.cornslice_depth[24 * corners + slice_sorted] >= togo2_limit:  # precheck speeds up the computation
//                 return

//             u_edges = self.co_cube.u_edges
//             d_edges = self.co_cube.d_edges
//             for m in self.sofar_phase1:
//                 u_edges = mv.u_edges_move[18 * u_edges + m]
//                 d_edges = mv.d_edges_move[18 * d_edges + m]
//             ud_edges = coord.u_edges_plus_d_edges_to_ud_edges[24 * u_edges + d_edges % 24]

//             dist2 = self.co_cube.get_depth_phase2(corners, ud_edges)
//             for togo2 in range(dist2, togo2_limit):  # do not use more than togo2_limit - 1 moves in phase 2
//                 self.sofar_phase2 = []
//                 self.phase2_done = False
//                 self.search_phase2(corners, ud_edges, slice_sorted, dist2, togo2)
//                 if self.phase2_done:  # solution already found
//                     break

//         else:
//             for m in Move:
//                 # dist = 0 means that we are already are in the subgroup H. If there are less than 5 moves left
//                 # this forces all remaining moves to be phase 2 moves. So we can forbid these at the end of phase 1
//                 # and generate these moves in phase 2.
//                 if dist == 0 and togo_phase1 < 5 and m in [Move.U1, Move.U2, Move.U3, Move.R2,
//                                                            Move.F2, Move.D1, Move.D2, Move.D3,
//                                                            Move.L2, Move.B2]:
//                     continue

//                 if len(self.sofar_phase1) > 0:
//                     diff = self.sofar_phase1[-1] // 3 - m // 3
//                     if diff in [0, 3]:  # successive moves: on same face or on same axis with wrong order
//                         continue

//                 flip_new = mv.flip_move[18 * flip + m]  # N_MOVE = 18
//                 twist_new = mv.twist_move[18 * twist + m]
//                 slice_sorted_new = mv.slice_sorted_move[18 * slice_sorted + m]

//                 flipslice = 2048 * (slice_sorted_new // 24) + flip_new  # N_FLIP * (slice_sorted // N_PERM_4) + flip
//                 classidx = sy.flipslice_classidx[flipslice]
//                 sym = sy.flipslice_sym[flipslice]
//                 dist_new_mod3 = pr.get_flipslice_twist_depth3(2187 * classidx + sy.twist_conj[(twist_new << 4) + sym])
//                 dist_new = pr.distance[3 * dist + dist_new_mod3]
//                 if dist_new >= togo_phase1:  # impossible to reach subgroup H in togo_phase1 - 1 moves
//                     continue

//                 self.sofar_phase1.append(m)
//                 self.search(flip_new, twist_new, slice_sorted_new, dist_new, togo_phase1 - 1)
//                 self.sofar_phase1.pop(-1)
