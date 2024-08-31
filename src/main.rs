extern crate solvers;
use solvers::thistlethwaite::{LookupTable, G1, G2, G3Pochmann, G4, Stage};
use solvers::cube::{Cube, Move};
use solvers::cube::Move::*;

fn solve_stage<'a, T: Stage<'a>>(cube: &mut Cube, table: LookupTable<T>) -> Vec<Move> {
    let mut solution = Vec::new();
    let mut steps = table.eval(&cube);
    
    while steps > 0 {
        for turn in T::MOVE_POOL {
            let mut temp = cube.clone();
            temp.turn(turn);
            let new_steps = table.eval(&temp);
            if new_steps < steps {
                cube.turn(turn);
                steps = new_steps;
                solution.push(*turn);
                break
            }
        }
    }

    return solution
}

fn direct_lookup(cube: &Cube) -> Vec<Move> {
    let filepaths = [
        "./solvers/src/thistlethwaite/data/g1.dat",
        "./solvers/src/thistlethwaite/data/g2.dat",
        "./solvers/src/thistlethwaite/data/g3.dat",
        "./solvers/src/thistlethwaite/data/g4.dat",
    ];
    
    let table1 = LookupTable::<G1>::new(Some(filepaths[0]));
    let table2 = LookupTable::<G2>::new(Some(filepaths[1]));
    let table3 = LookupTable::<G3Pochmann>::new(Some(filepaths[2]));
    let table4 = LookupTable::<G4>::new(Some(filepaths[3]));

    let mut cube = cube.clone();
    let mut solution: Vec<Move> = Vec::new();
    solution.append(&mut solve_stage(&mut cube, table1));
    solution.append(&mut solve_stage(&mut cube, table2));
    solution.append(&mut solve_stage(&mut cube, table3));
    solution.append(&mut solve_stage(&mut cube, table4));
    solution
}

fn main() {
    // Simple Thisletwaithe using lookuptable
    let scramble = vec![Rp, U2, R2, Dp, Lp, Bp, L2, Up, R2, D2, R, B2, Lp, D2, Rp, F2, B2, R, F];
    let mut cube = Cube::new();
    cube.apply_moves(&scramble);
    
    let solution = direct_lookup(&cube);

    let mut cube = Cube::new();
    cube.apply_moves(&scramble).apply_moves(&solution);
    assert!(cube.is_solved());
}