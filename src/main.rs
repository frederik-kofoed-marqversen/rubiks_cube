extern crate cube;
extern crate solvers;

use cube::{Cube, Rng, Moveable};
use solvers::Solver;
// use solvers::thistlethwaite::{LookupTableSolver, ThistlethwaiteTables, BFSSolver, BDBFSSolver};
use solvers::kociemba::{KociembaSolver, KociembaTables, SearchContext};
// use cube::Move::*;

fn main() {
    let mut rng = Rng::new();
    
    // let scramble = vec![Rp, U2, R2, Dp, Lp, Bp, L2, Up, R2, D2, R, B2, Lp, D2, Rp, F2, B2, R, F];
    
    // // Initialize scrambled cube
    // let mut cube = Cube::solved();
    // cube.apply_moves(&scramble);
    
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

    // // Build Kociemba tables
    // let tables = KociembaTables::build();
    // tables.save(KociembaTables::DEFAULT_PATH).expect("Failed to save Kociemba tables");

    // Kociemba's algorithm
    println!("\n--- Testing Kociemba's Algorithm ---");
    let tables = KociembaTables::load_or_build(KociembaTables::DEFAULT_PATH)
        .expect("Failed to initialize Kociemba tables");
    let solver = KociembaSolver::new(&tables);
    
    let timeout = std::time::Duration::from_secs_f32(0.001);
    for _ in 0..10 {
        let mut cube = Cube::new_random(&mut rng);
        
        let ctx = SearchContext::new(cube, Some(timeout));
        let solution = solver.run(ctx);
        let moves = solution.expect("No solution found by Kociemba's algorithm.");
        
        cube.apply_moves(&moves);
        assert!(cube.is_solved());
        println!("Cube solved in {} moves!", moves.len());
    }
}