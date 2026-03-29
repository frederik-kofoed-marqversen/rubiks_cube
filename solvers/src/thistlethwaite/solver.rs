use super::{LookupTable, Stage, G1, G2, G3Pochmann, G4};
use crate::cube::{Cube, Move};
use crate::solver::Solver;

pub struct ThistlethwaiteSolver {
    table1: Option<LookupTable<G1>>,
    table2: Option<LookupTable<G2>>,
    table3: Option<LookupTable<G3Pochmann>>,
    table4: Option<LookupTable<G4>>,
}

impl ThistlethwaiteSolver {
    /// Create a new uninitialized solver
    /// Call `with_tables()`, `load_tables()`, or `build_tables()` before solving
    pub fn new() -> Self {
        Self {
            table1: None,
            table2: None,
            table3: None,
            table4: None,
        }
    }
    
    /// Default data directory for precomputed tables (in target/ to avoid committing to repo)
    pub const DEFAULT_DATA_DIR: &'static str = "./target/data/thistlethwaite";
    
    fn table_paths(data_dir: &str) -> [String; 4] {
        [
            format!("{}/g1.dat", data_dir),
            format!("{}/g2.dat", data_dir),
            format!("{}/g3.dat", data_dir),
            format!("{}/g4.dat", data_dir),
        ]
    }
    
    fn solve_stage<'a, T: Stage<'a>>(&self, cube: &mut Cube, table: &LookupTable<T>) -> Vec<Move> {
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
                    break;
                }
            }
        }
        
        solution
    }
}

impl Solver for ThistlethwaiteSolver {
    fn solve(&self, cube: &Cube) -> Vec<Move> {
        assert!(self.is_ready(), 
            "Solver not ready - call load_tables(), build_tables(), or with_tables() first");
        
        let mut cube = cube.clone();
        let mut solution = Vec::new();
        
        solution.append(&mut self.solve_stage(&mut cube, self.table1.as_ref().unwrap()));
        solution.append(&mut self.solve_stage(&mut cube, self.table2.as_ref().unwrap()));
        solution.append(&mut self.solve_stage(&mut cube, self.table3.as_ref().unwrap()));
        solution.append(&mut self.solve_stage(&mut cube, self.table4.as_ref().unwrap()));
        
        solution
    }
    
    fn build_tables(&mut self) -> Result<(), std::io::Error> {
        println!("Building G1 table...");
        self.table1 = Some(LookupTable::<G1>::build());
        
        println!("Building G2 table...");
        self.table2 = Some(LookupTable::<G2>::build());
        
        println!("Building G3 table...");
        self.table3 = Some(LookupTable::<G3Pochmann>::build());
        
        println!("Building G4 table...");
        self.table4 = Some(LookupTable::<G4>::build());
        
        Ok(())
    }
    
    fn load_tables(&mut self, data_dir: &str) -> Result<(), std::io::Error> {
        let paths = Self::table_paths(data_dir);
        
        self.table1 = Some(LookupTable::<G1>::load(&paths[0])?);
        self.table2 = Some(LookupTable::<G2>::load(&paths[1])?);
        self.table3 = Some(LookupTable::<G3Pochmann>::load(&paths[2])?);
        self.table4 = Some(LookupTable::<G4>::load(&paths[3])?);
        
        Ok(())
    }
    
    fn save_tables(&self, data_dir: &str) -> Result<(), std::io::Error> {
        let paths = Self::table_paths(data_dir);
        
        self.table1.as_ref().unwrap().save(&paths[0])?;
        self.table2.as_ref().unwrap().save(&paths[1])?;
        self.table3.as_ref().unwrap().save(&paths[2])?;
        self.table4.as_ref().unwrap().save(&paths[3])?;
        
        Ok(())
    }
    
    fn is_ready(&self) -> bool {
        self.table1.is_some() && 
        self.table2.is_some() && 
        self.table3.is_some() && 
        self.table4.is_some()
    }
    
    fn name(&self) -> &str {
        "Thistlethwaite (Greedy)"
    }
}