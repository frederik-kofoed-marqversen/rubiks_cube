use super::lookup_table::LookupTable;
use super::stages::{G1, G2, G3Pochmann, G4};
use std::io;

pub struct ThistlethwaiteTables {
    pub(super) g1: LookupTable<G1>,
    pub(super) g2: LookupTable<G2>,
    pub(super) g3: LookupTable<G3Pochmann>,
    pub(super) g4: LookupTable<G4>,
}

impl ThistlethwaiteTables {
    /// Default data directory for precomputed tables (in target/ to avoid committing to repo)
    pub const DEFAULT_DIR: &'static str = "./target/data/thistlethwaite";
    
    /// Build all lookup tables from scratch
    /// This takes several seconds but only needs to be done once
    pub fn build() -> Self {
        println!("Building G1 table...");
        let g1 = LookupTable::<G1>::build();
        
        println!("Building G2 table...");
        let g2 = LookupTable::<G2>::build();
        
        println!("Building G3 table...");
        let g3 = LookupTable::<G3Pochmann>::build();
        
        println!("Building G4 table...");
        let g4 = LookupTable::<G4>::build();
        
        Self { g1, g2, g3, g4 }
    }
    
    /// Load precomputed tables from directory
    /// Returns error if files don't exist or are invalid
    pub fn load(data_dir: &str) -> Result<Self, io::Error> {
        let paths = Self::table_paths(data_dir);
        
        let g1 = LookupTable::<G1>::load(&paths[0])?;
        let g2 = LookupTable::<G2>::load(&paths[1])?;
        let g3 = LookupTable::<G3Pochmann>::load(&paths[2])?;
        let g4 = LookupTable::<G4>::load(&paths[3])?;
        
        Ok(Self { g1, g2, g3, g4 })
    }
    
    /// Try to load tables from directory, build from scratch if not found
    /// This is the recommended initialization method
    pub fn load_or_build(data_dir: &str) -> Result<Self, io::Error> {
        match Self::load(data_dir) {
            Ok(tables) => {
                println!("Loaded tables from {}", data_dir);
                Ok(tables)
            }
            Err(_) => {
                println!("Tables not found in {}, building from scratch...", data_dir);
                let tables = Self::build();
                tables.save(data_dir)?;
                println!("Saved tables to {}", data_dir);
                Ok(tables)
            }
        }
    }
    
    /// Save tables to directory
    /// Creates directory if it doesn't exist
    pub fn save(&self, data_dir: &str) -> Result<(), io::Error> {
        let paths = Self::table_paths(data_dir);
        
        self.g1.save(&paths[0])?;
        self.g2.save(&paths[1])?;
        self.g3.save(&paths[2])?;
        self.g4.save(&paths[3])?;
        
        Ok(())
    }
    
    fn table_paths(data_dir: &str) -> [String; 4] {
        [
            format!("{}/g1.dat", data_dir),
            format!("{}/g2.dat", data_dir),
            format!("{}/g3.dat", data_dir),
            format!("{}/g4.dat", data_dir),
        ]
    }
}
