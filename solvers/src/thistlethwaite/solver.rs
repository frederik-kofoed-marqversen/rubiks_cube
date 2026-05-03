use super::lookup_table::LookupTable;
use super::stages::{G3Pochmann, Stage, G1, G2, G4};
use super::ThistlethwaiteTables;
use super::cube::{Cube, Move, Moveable};
use super::Solver;
use std::collections::{HashMap, VecDeque};

pub struct LookupTableSolver {
    tables: ThistlethwaiteTables,
}

impl LookupTableSolver {
    pub fn new(tables: ThistlethwaiteTables) -> Self {
        Self { tables }
    }

    fn solve_stage<T: Stage>(&self, cube: &mut Cube, table: &LookupTable<T>) -> Vec<Move> {
        let mut solution = Vec::new();
        let mut steps = table.eval(&cube);

        while steps > 0 {
            for &turn in T::MOVE_POOL {
                let mut temp = cube.clone();
                temp.turn(turn);
                let new_steps = table.eval(&temp);
                if new_steps < steps {
                    cube.turn(turn);
                    steps = new_steps;
                    solution.push(turn);
                    break;
                }
            }
        }

        solution
    }
}

impl Solver for LookupTableSolver {
    fn solve(&self, mut cube: Cube) -> Vec<Move> {
        let mut solution = Vec::new();

        solution.append(&mut self.solve_stage(&mut cube, &self.tables.g1));
        solution.append(&mut self.solve_stage(&mut cube, &self.tables.g2));
        solution.append(&mut self.solve_stage(&mut cube, &self.tables.g3));
        solution.append(&mut self.solve_stage(&mut cube, &self.tables.g4));

        solution
    }

    fn name(&self) -> &str {
        "Thistlethwaite's Algorithm (greedy lookup table)"
    }
}

// BFS expansion for one frontier, returns connection to reverse frontier if found
fn expand_frontier<T: Stage>(
    queue: &mut VecDeque<Cube>,
    this_map: &mut HashMap<u32, (u32, Option<Move>)>,
    reverse_map: &HashMap<u32, (u32, Option<Move>)>,
) -> Option<u32> {
    if let Some(current) = queue.pop_front() {
        let current_id: u32 = T::id(&current);

        for &turn in T::MOVE_POOL {
            let mut next = current;
            next.turn(turn);
            let next_id = T::id(&next);

            if !this_map.contains_key(&next_id) {
                this_map.insert(next_id, (current_id, Some(turn)));
                queue.push_back(next);

                if reverse_map.contains_key(&next_id) {
                    return Some(next_id);
                }
            }
        }
    }
    None
}

// Reconstruct the path from start to goal using the forward and backward maps
fn reconstruct_path<T: Stage>(
    connection: u32,
    forward_map: &HashMap<u32, (u32, Option<Move>)>,
    backward_map: &HashMap<u32, (u32, Option<Move>)>,
) -> Vec<Move> {
    let mut path = Vec::new();

    // Reconstruct forward path
    let mut current_id = connection;
    while let Some((previous_id, Some(mv))) = forward_map.get(&current_id) {
        path.push(*mv);
        current_id = *previous_id;
    }
    path.reverse();

    // Reconstruct backward path
    let mut current_id = connection;
    while let Some((previous_id, Some(mv))) = backward_map.get(&current_id) {
        path.push(mv.inverse());
        current_id = *previous_id;
    }

    path
}

pub struct BDBFSSolver;

impl BDBFSSolver {
    pub fn new() -> Self {
        Self
    }

    // Bi-directional BFS solver for a single stage
    fn solve_stage<T: Stage>(cube: &mut Cube) -> Vec<Move> {
        let solved = Cube::solved();
        let start_id = T::id(cube);
        let goal_id = T::id(&solved);

        if start_id == goal_id {
            return vec![]; // Already in goal class!
        }

        let mut forward: HashMap<u32, (u32, Option<Move>)> = HashMap::new();
        let mut backward: HashMap<u32, (u32, Option<Move>)> = HashMap::new();

        forward.insert(start_id, (start_id, None));
        backward.insert(goal_id, (goal_id, None));

        // Two queues
        let mut forward_queue = VecDeque::from([*cube]);
        let mut backward_queue = VecDeque::from([solved]);
        let connection;
        loop {
            // Expand forward frontier
            if let Some(c) = expand_frontier::<T>(&mut forward_queue, &mut forward, &backward) {
                connection = c;
                break;
            }

            // Expand backward frontier
            if let Some(c) = expand_frontier::<T>(&mut backward_queue, &mut backward, &forward) {
                connection = c;
                break;
            }
        }

        let solution = reconstruct_path::<T>(connection, &forward, &backward);
        cube.apply_moves(&solution);
        return solution;
    }
}

impl Solver for BDBFSSolver {
    fn solve(&self, mut cube: Cube) -> Vec<Move> {
        let mut solution = Vec::new();

        solution.append(&mut Self::solve_stage::<G1>(&mut cube));
        solution.append(&mut Self::solve_stage::<G2>(&mut cube));
        solution.append(&mut Self::solve_stage::<G3Pochmann>(&mut cube));
        solution.append(&mut Self::solve_stage::<G4>(&mut cube));
        solution.append(&mut Self::solve_stage::<G4>(&mut cube));

        solution
    }

    fn name(&self) -> &str {
        "Thistlethwaite's Algorithm (bidirectional BFS)"
    }
}

pub struct BFSSolver;

impl BFSSolver {
    pub fn new() -> Self {
        Self
    }

    // Bi-directional BFS solver for a single stage
    fn solve_stage<T: Stage>(cube: &mut Cube) -> Vec<Move> {
        let solved = Cube::solved();
        let start_id = T::id(cube);
        let goal_id = T::id(&solved);

        if start_id == goal_id {
            return vec![]; // Already in goal class!
        }

        let mut map: HashMap<u32, (u32, Option<Move>)> = HashMap::new();
        let mut goal_map: HashMap<u32, (u32, Option<Move>)> = HashMap::new();

        map.insert(start_id, (start_id, None));
        goal_map.insert(goal_id, (goal_id, None));

        let mut queue = VecDeque::from([*cube]);
        loop {
            // Expand forward frontier
            if expand_frontier::<T>(&mut queue, &mut map, &goal_map).is_some() {
                break;
            }
        }

        let solution = reconstruct_path::<T>(goal_id, &map, &goal_map);
        cube.apply_moves(&solution);
        return solution;
    }
}

impl Solver for BFSSolver {
    fn solve(&self, mut cube: Cube) -> Vec<Move> {
        let mut solution = Vec::new();

        solution.append(&mut Self::solve_stage::<G1>(&mut cube));
        solution.append(&mut Self::solve_stage::<G2>(&mut cube));
        solution.append(&mut Self::solve_stage::<G3Pochmann>(&mut cube));
        solution.append(&mut Self::solve_stage::<G4>(&mut cube));
        solution.append(&mut Self::solve_stage::<G4>(&mut cube));

        solution
    }

    fn name(&self) -> &str {
        "Thistlethwaite's Algorithm (BFS)"
    }
}
