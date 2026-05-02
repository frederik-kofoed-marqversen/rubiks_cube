use super::cube::{Cube, Move, MOVES};
use super::indexers::*;
use super::KociembaTables;

pub trait PhaseState: Copy {
    fn is_solved(&self) -> bool;
    fn turn(&self, mv: Move, tables: &KociembaTables) -> Self;
    fn heuristic(&self, tables: &KociembaTables) -> u8;
}

#[derive(Clone, Copy)]
pub struct Phase1State {
    pub eo: usize,
    pub co: usize,
    pub es: usize,
}

impl Phase1State {
    pub fn from_cube(cube: &Cube) -> Self {
        Self {
            eo: EdgeOrientationIndexer::to_index(cube),
            co: CornerOrientationIndexer::to_index(cube),
            es: ESliceIndexer::to_index(cube),
        }
    }
}

impl PhaseState for Phase1State {
    fn is_solved(&self) -> bool {
        self.eo == 0 && self.co == 0 && self.es / 24 == 0
    }

    fn turn(&self, mv: Move, tables: &KociembaTables) -> Self {
        Self {
            eo: tables.eo_move.get(self.eo, mv) as usize,
            co: tables.co_move.get(self.co, mv) as usize,
            es: tables.es_move.get(self.es, mv) as usize,
        }
    }

    fn heuristic(&self, tables: &KociembaTables) -> u8 {
        let h1 = tables.eo_es_prune.get(self.eo, self.es);
        let h2 = tables.co_es_prune.get(self.co, self.es);
        h1.max(h2)
    }
}

#[derive(Clone, Copy)]
pub struct Phase2State {
    pub cp: usize,
    pub es: usize,
    pub ue: usize,
    pub de: usize,
}

impl Phase2State {
    pub fn from_cube(cube: &Cube) -> Self {
        Self {
            cp: CornerPermutationIndexer::to_index(cube),
            es: ESliceIndexer::to_index(cube),
            ue: UEdgeIndexer::to_index(cube),
            de: DEdgeIndexer::to_index(cube),
        }
    }

    pub fn apply_moves(&self, moves: &[Move], tables: &KociembaTables) -> Self {
        let mut state = *self;
        for &mv in moves {
            state = state.turn(mv, tables);
        }
        state
    }
}

impl PhaseState for Phase2State {
    fn is_solved(&self) -> bool {
        self.cp == 0 && self.es == 0 && self.ue == 0 && self.de == 0
    }

    fn turn(&self, mv: Move, tables: &KociembaTables) -> Self {
        Self {
            cp: tables.cp_move.get(self.cp, mv) as usize,
            es: tables.es_move.get(self.es, mv) as usize,
            ue: tables.ue_move.get(self.ue, mv) as usize,
            de: tables.de_move.get(self.de, mv) as usize,
        }
    }

    fn heuristic(&self, tables: &KociembaTables) -> u8 {
        let h1 = tables.cp_es_prune.get(self.cp, self.es);
        let h2 = tables.cp_ue_prune.get(self.cp, self.ue);
        let h3 = tables.cp_de_prune.get(self.cp, self.de);
        h1.max(h2).max(h3)
    }
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

pub fn solve_phase<S: PhaseState>(
    start: S,
    tables: &KociembaTables,
    max_depth: u8,
) -> Vec<Move> {
    let mut bound = start.heuristic(tables);

    if bound == 0 {
        return Vec::new();
    }

    loop {
        if bound > max_depth {
            panic!("Failed to find a solution within {} moves", max_depth);
        }

        let mut path = Vec::new();
        if dfs(start, 0, bound, &mut path, tables) {
            return path;
        }

        bound += 1;
    }
}

fn dfs<S: PhaseState>(
    state: S,
    depth: u8,
    bound: u8,
    path: &mut Vec<Move>,
    tables: &KociembaTables,
) -> bool {
    if depth + state.heuristic(tables) > bound {
        return false;
    }
    if state.is_solved() {
        return true;
    }

    for mv in MOVES {
        if !is_valid_move(path.last(), mv) {
            continue;
        }

        let next_state = state.turn(mv, tables);
        path.push(mv);
        if dfs(next_state, depth + 1, bound, path, tables) {
            return true;
        }
        path.pop();
    }

    false
}

// Phase 2 solver!
// def search(self, flip, twist, slice_sorted, dist, togo_phase1):
//         # ##############################################################################################################
//         if self.terminated.is_set():
//             return
//         ################################################################################################################
//         if togo_phase1 == 0:  # phase 1 solved -> solve phase 2

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

//  def search_phase2(self, corners, ud_edges, slice_sorted, dist, togo_phase2):
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

#[cfg(test)]
mod tests {
    use super::*;
    use super::Move::*;
    use super::super::cube::Cube;
    use std::sync::LazyLock;

    static TABLES: LazyLock<KociembaTables> = LazyLock::new(KociembaTables::build);

    #[test]
    fn phase1_solved_cube() {
        let cube = Cube::solved();
        let state = Phase1State::from_cube(&cube);
        let solution = solve_phase(state, &TABLES, 20);
        assert_eq!(solution.len(), 0, "Solved cube needs 0 moves");
    }

    #[test]
    fn phase1_simple_scramble() {
        let mut cube = Cube::solved();
        cube.turn(F);

        let state = Phase1State::from_cube(&cube);
        let solution = solve_phase(state, &TABLES, 20);

        // Verify solution
        cube.apply_moves(&solution);
        let final_state = Phase1State::from_cube(&cube);
        assert!(final_state.is_solved(), "Phase 1 should reach goal state");
    }

    #[test]
    fn phase1_short_scramble() {
        let scramble = vec![R, U, Rp, Up];
        let mut cube = Cube::solved();
        cube.apply_moves(&scramble);

        let state = Phase1State::from_cube(&cube);
        let solution = solve_phase(state, &TABLES, 20);
        assert!(solution.len() <= 10, "Short scramble should solve quickly");

        // Verify solution
        cube.apply_moves(&solution);
        let final_state = Phase1State::from_cube(&cube);
        assert!(final_state.is_solved(), "Solution should reach Phase 1 goal");
    }
}
