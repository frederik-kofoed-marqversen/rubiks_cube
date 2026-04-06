use super::cube::{Cube, Move, MOVES};
use super::indexers::{CornerOrientationIndexer, ESliceIndexer, EdgeOrientationIndexer, Indexer};
use super::KociembaTables;

pub struct Phase1Solver;

impl Phase1Solver {
    pub fn solve(cube: &Cube, tables: &KociembaTables) -> Vec<Move> {
        let start = Phase1State::from_cube(cube);
        let mut bound = Self::heuristic(&start, tables);

        if bound == 0 {
            return Vec::new();
        }

        loop {
            if bound > 20 {
                panic!("Failed to find a solution in Phase 1 within 20 moves");
            }

            let mut path = Vec::new();
            if Self::dfs(start, 0, bound, tables, &mut path) {
                return path;
            }

            bound += 1;
        }
    }

    fn dfs(
        state: Phase1State,
        depth: u8,
        bound: u8,
        tables: &KociembaTables,
        path: &mut Vec<Move>,
    ) -> bool {
        if depth + Self::heuristic(&state, tables) > bound {
            return false;
        }
        if state.is_solved() {
            return true;
        }

        for mv in MOVES {
            if !Self::is_valid_move(path.last(), mv) {
                continue;
            }

            let next_state = state.apply_move(mv, tables);
            path.push(mv);
            if Self::dfs(next_state, depth + 1, bound, tables, path) {
                return true;
            }
            path.pop();
        }

        false
    }

    fn heuristic(state: &Phase1State, tables: &KociembaTables) -> u8 {
        // The heuristic is the maximum of the two pruning tables
        let h1 = tables.eo_es_prune.get(state.eo, state.es);
        let h2 = tables.co_es_prune.get(state.co, state.es);
        h1.max(h2)
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
}

#[derive(Clone, Copy)]
struct Phase1State {
    eo: usize,
    es: usize,
    co: usize,
}

impl Phase1State {
    fn from_cube(cube: &Cube) -> Self {
        Self {
            eo: EdgeOrientationIndexer::to_index(cube),
            es: ESliceIndexer::to_index(cube),
            co: CornerOrientationIndexer::to_index(cube),
        }
    }

    fn apply_move(&self, mv: Move, tables: &KociembaTables) -> Self {
        let eo = tables.eo_move.get(self.eo, mv) as usize;
        let es = tables.es_move.get(self.es, mv) as usize;
        let co = tables.co_move.get(self.co, mv) as usize;
        Self { eo, es, co }
    }

    fn is_solved(&self) -> bool {
        self.eo == 0 && self.es == 0 && self.co == 0
    }
}

#[cfg(test)]
mod tests {
    use super::Move::*;
    use super::*;
    use std::sync::LazyLock;

    static TABLES: LazyLock<KociembaTables> = LazyLock::new(KociembaTables::build);

    #[test]
    fn phase1_solved_cube() {
        let cube = Cube::solved();
        let solution = Phase1Solver::solve(&cube, &TABLES);
        assert_eq!(solution.len(), 0, "Solved cube needs 0 moves");
    }

    #[test]
    fn phase1_simple_scramble() {
        let mut cube = Cube::solved();
        cube.turn(F);

        let solution = Phase1Solver::solve(&cube, &TABLES);

        // Verify solution
        cube.apply_moves(&solution);
        let state = Phase1State::from_cube(&cube);
        assert!(state.is_solved(), "Phase 1 should reach goal state");
    }

    #[test]
    fn phase1_short_scramble() {
        let scramble = vec![R, U, Rp, Up];
        let mut cube = Cube::solved();
        cube.apply_moves(&scramble);

        let solution = Phase1Solver::solve(&cube, &TABLES);
        assert!(solution.len() <= 10, "Short scramble should solve quickly");

        // Verify solution
        cube.apply_moves(&solution);
        let state = Phase1State::from_cube(&cube);
        assert!(state.is_solved(), "Solution should reach Phase 1 goal");
    }
}
