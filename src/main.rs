extern crate solvers;
use solvers::thistlethwaite::{LookupTable, G1, G2, G3, G4, Stage};
use solvers::cube::{Cube, Move};
use Move::*;

fn main() {
    /* // Solve stage
    let scramble = vec![Rp, U2, R2, Dp, Lp, Bp, L2, Up, R2, D2, R, B2, Lp, D2, Rp, F2, B2, R, F];
    let mut cube = Cube::new();
    for turn in scramble {
        cube.turn(&turn);
    }
    let mut solution = Vec::new();
    
    let table = LookupTable::<G1>::new(
        Some("./solvers/src/thistlethwaite/data/g1.dat"
    ));
    let mut steps = table.eval(&cube);
    dbg!(steps);
    while steps > 0 {
        for turn in G1::MOVE_POOL {
            let mut temp = cube.clone();
            temp.turn(turn);
            let new_steps = table.eval(&temp);
            if new_steps < steps {
                cube.turn(turn);
                steps = new_steps;
                solution.push(turn);
                break
            }
        }
    }

    let table = LookupTable::<G2>::new(
        Some("./solvers/src/thistlethwaite/data/g2.dat"
    ));
    let mut steps = table.eval(&cube);
    dbg!(steps);
    while steps > 0 {
        for turn in G1::MOVE_POOL {
            let mut temp = cube.clone();
            temp.turn(turn);
            let new_steps = table.eval(&temp);
            if new_steps < steps {
                cube.turn(turn);
                steps = new_steps;
                solution.push(turn);
                break
            }
        }
    }
    dbg!(&solution); */

    let mut cube = Cube::new();
    cube.turn(&Move::U2);
    cube.turn(&Move::D2);

    let table1 = LookupTable::<G1>::new(
        Some("./solvers/src/thistlethwaite/data/g1.dat"
    ));
    dbg!(table1.eval(&cube));
    
    let table2 = LookupTable::<G2>::new(
        Some("./solvers/src/thistlethwaite/data/g2.dat"
    ));
    dbg!(table2.eval(&cube));
    
    let table3 = LookupTable::<G3>::new(
        Some("./solvers/src/thistlethwaite/data/g3.dat"
    ));
    dbg!(table3.eval(&cube));

    let table4 = LookupTable::<G4>::new(
        Some("./solvers/src/thistlethwaite/data/g4.dat"
    ));
    dbg!(table4.eval(&cube));
}