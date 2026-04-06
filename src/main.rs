extern crate solvers;
use solvers::Solver;
// use solvers::thistlethwaite::{LookupTableSolver, ThistlethwaiteTables, BFSSolver, BDBFSSolver};
use solvers::kociemba::{KociembaSolver, KociembaTables};
use solvers::cube::Cube;
use solvers::cube::Move::*;

fn main() {
    let scramble = vec![Rp, U2, R2, Dp, Lp, Bp, L2, Up, R2, D2, R, B2, Lp, D2, Rp, F2, B2, R, F];
    
    // Initialize scrambled cube
    let mut cube = Cube::solved();
    cube.apply_moves(&scramble);
    
    // // Initialise solver
    // // // let tables = ThistlethwaiteTables::load_or_build(ThistlethwaiteTables::DEFAULT_DIR)
    // // //     .expect("Failed to initialize tables");
    // // let tables = ThistlethwaiteTables::build();
    // // let solver = LookupTableSolver::new(tables);
    // let solver = BDBFSSolver::new();
    
    // // Solve
    // let solution = solver.solve(cube);
    // println!("Solution found by {}: {} moves", solver.name(), solution.len());
    // println!("Solution: {:?}", solution);
    
    // // Verify
    // let mut cube = Cube::solved();
    // cube.apply_moves(&scramble).apply_moves(&solution);
    // assert!(cube.is_solved());
    // println!("✓ Cube solved successfully!");

    // Kociemba's algorithm
    println!("\n--- Testing Kociemba's Algorithm ---");
    // let tables = KociembaTables::load_or_build(KociembaTables::DEFAULT_PATH)
    //     .expect("Failed to initialize Kociemba tables");
    let tables = KociembaTables::build();
    let solver = KociembaSolver::new(tables);
    
    let mut scrambled_cube = Cube::solved();
    scrambled_cube.apply_moves(&scramble);
    
    let solution = solver.solve(scrambled_cube);
    println!("Kociemba's solution: {} moves", solution.len());
    
    // Verify Kociemba solution
    let mut cube = Cube::solved();
    cube.apply_moves(&scramble).apply_moves(&solution);
    assert!(cube.is_solved());
    println!("✓ Kociemba solved successfully!");
}