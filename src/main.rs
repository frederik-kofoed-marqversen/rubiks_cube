extern crate solvers;
use solvers::Solver;
use solvers::thistlethwaite::ThistlethwaiteSolver;
use solvers::cube::Cube;
use solvers::cube::Move::*;

fn main() {
    let scramble = vec![Rp, U2, R2, Dp, Lp, Bp, L2, Up, R2, D2, R, B2, Lp, D2, Rp, F2, B2, R, F];
    
    // Initialize scrambled cube
    let mut cube = Cube::new();
    cube.apply_moves(&scramble);
    
    // Create solver with tables (will load from disk or build if not found)
    let solver = ThistlethwaiteSolver::new()
        .with_tables(ThistlethwaiteSolver::DEFAULT_DATA_DIR)
        .expect("Failed to initialize solver tables");
    
    // Solve
    let solution = solver.solve(&cube);
    println!("Solution found by {}: {} moves", solver.name(), solution.len());
    println!("Solution: {:?}", solution);
    
    // Verify
    let mut cube = Cube::new();
    cube.apply_moves(&scramble).apply_moves(&solution);
    assert!(cube.is_solved());
    println!("✓ Cube solved successfully!");
}