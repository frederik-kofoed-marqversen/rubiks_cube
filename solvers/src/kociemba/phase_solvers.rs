use super::cube::{Move, MOVES};
use super::coord_cube::CoordinateCube;
use super::KociembaTables;

pub struct Phase1Solver;

impl Phase1Solver {
    pub fn solve(start: CoordinateCube, tables: &KociembaTables) -> (Vec<Move>, CoordinateCube) {
        let mut bound = Self::heuristic(&start, tables);

        if bound == 0 {
            return (Vec::new(), start);
        }

        loop {
            if bound > 20 {
                panic!("Failed to find a solution in Phase 1 within 20 moves");
            }

            let mut path = Vec::new();
            if let Some(result) = Self::dfs(start, 0, bound, &mut path, tables) {
                return (path, result);
            }

            bound += 1;
        }
    }

    fn is_solved(state: &CoordinateCube) -> bool {
        // divide by 24 to extract ESlice combination
        state.eo == 0 && state.co == 0 && state.es / 24 == 0
    }
    
    fn turn(state: &CoordinateCube, mv: Move, tables: &KociembaTables) -> CoordinateCube {
        CoordinateCube {
            eo: tables.eo_move.get(state.eo, mv) as usize,
            co: tables.co_move.get(state.co, mv) as usize,
            cp: tables.cp_move.get(state.cp, mv) as usize,
            es: tables.es_move.get(state.es, mv) as usize,
            ue: tables.ue_move.get(state.ue, mv) as usize,
            de: tables.de_move.get(state.de, mv) as usize,
        }
    }

    fn heuristic(state: &CoordinateCube, tables: &KociembaTables) -> u8 {
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

    fn dfs(
        state: CoordinateCube,
        depth: u8,
        bound: u8,
        path: &mut Vec<Move>,
        tables: &KociembaTables,
    ) -> Option<CoordinateCube> {
        if depth + Self::heuristic(&state, tables) > bound {
            return None;
        }
        if Self::is_solved(&state) {
            return Some(state);

        }

        for mv in MOVES {
            if !Self::is_valid_move(path.last(), mv) {
                continue;
            }

            let next_state = Self::turn(&state, mv, tables);
            path.push(mv);
            if let Some(result) = Self::dfs(next_state, depth + 1, bound, path, tables) {
                return Some(result);
            }
            path.pop();
        }

        None
    }
}

pub struct Phase2Solver;

impl Phase2Solver {
    pub fn solve(start: CoordinateCube, tables: &KociembaTables) -> Vec<Move> {
        unimplemented!()
    }

    fn is_solved(state: &CoordinateCube) -> bool {
        state.cp == 0 && state.es == 0 && state.ue == 0 && state.de == 0
    }

    fn turn(state: &CoordinateCube, mv: Move, tables: &KociembaTables) -> CoordinateCube {
        CoordinateCube {
            eo: state.eo,
            co: state.co,
            cp: tables.cp_move.get(state.cp, mv) as usize,
            es: state.es,
            ue: tables.ue_move.get(state.ue, mv) as usize,
            de: tables.de_move.get(state.de, mv) as usize,
        }
    }

    fn heuristic(state: &CoordinateCube, tables: &KociembaTables) -> u8 {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::Move::*;
    use super::super::cube::Cube;
    use std::sync::LazyLock;

    static TABLES: LazyLock<KociembaTables> = LazyLock::new(KociembaTables::build);

    #[test]
    fn phase1_solved_cube() {
        let cube = CoordinateCube::solved();
        let (solution, _) = Phase1Solver::solve(cube, &TABLES);
        assert_eq!(solution.len(), 0, "Solved cube needs 0 moves");
    }

    #[test]
    fn phase1_simple_scramble() {
        let mut cube = Cube::solved();
        cube.turn(F);

        let (solution, _) = Phase1Solver::solve(CoordinateCube::from_cube(&cube), &TABLES);

        // Verify solution
        cube.apply_moves(&solution);
        let state = CoordinateCube::from_cube(&cube);
        assert!(Phase1Solver::is_solved(&state), "Phase 1 should reach goal state");
    }

    #[test]
    fn phase1_short_scramble() {
        let scramble = vec![R, U, Rp, Up];
        let mut cube = Cube::solved();
        cube.apply_moves(&scramble);

        let (solution, _) = Phase1Solver::solve(CoordinateCube::from_cube(&cube), &TABLES);
        assert!(solution.len() <= 10, "Short scramble should solve quickly");

        // Verify solution
        cube.apply_moves(&solution);
        let state = CoordinateCube::from_cube(&cube);
        assert!(Phase1Solver::is_solved(&state), "Solution should reach Phase 1 goal");
    }
}
