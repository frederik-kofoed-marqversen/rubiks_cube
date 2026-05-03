use super::cube::{Move, MOVES};
use super::indexers::*;
use crate::math::update_distance_mod3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
struct MoveTable {
    data: Vec<u16>,
}

impl MoveTable {
    #[inline]
    pub fn get(&self, idx: usize, mv: Move) -> u16 {
        self.data[idx * 18 + mv as usize]
    }

    #[inline]
    fn set(&mut self, idx: usize, mv: Move, val: u16) {
        self.data[idx * 18 + mv as usize] = val;
    }

    pub fn build<I: CubeIndexer>() -> Self {
        let mut table = Self {
            data: vec![0; I::SIZE * 18],
        };
        for idx1 in 0..I::SIZE {
            for mv in MOVES {
                let mut cube = I::from_index(idx1);
                cube.turn(mv);
                table.set(idx1, mv, I::to_index(&cube) as u16);
            }
        }
        table
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct PruningTable {
    data: Vec<u32>,
}

impl PruningTable {
    #[inline]
    pub fn get(&self, index: usize, prev_distance: u32) -> u32 {
        let mod3 = self.get_mod3(index);
        update_distance_mod3(prev_distance, mod3)
    }

    #[inline]
    fn get_mod3(&self, index: usize) -> u32 {
        let u32_index = index >> 4;
        let bit_offset = (index & 0xF) << 1;
        let val = (self.data[u32_index] >> bit_offset) & 0b11; // mask to get 2 bits
        val
    }

    #[inline]
    fn set_mod3(&mut self, index: usize, val: u32) {
        let u32_index = index >> 4;
        let bit_offset = (index & 0xF) << 1;
        self.data[u32_index] &= !(0b11 << bit_offset); // clear the bits
        self.data[u32_index] |= (val % 3) << bit_offset; // set the new value
    }

    pub fn build(move_table: &MoveTable) -> Self {
        let width = move_table.data.len() / 18;
        let mut table = Self {
            data: vec![0xFFFFFFFF; width / 16 + 1],
        }; // Initialize all entries to 3 (0b11)
        
        let mut queue = std::collections::VecDeque::new();

        // Starting from solved state.
        // Assuming both indexers return 0 for the solved cube
        const SOLVED: usize = 0;
        table.set_mod3(SOLVED, 0); // Distance to solved state is 0
        queue.push_back(SOLVED);

        while let Some(index) = queue.pop_front() {
            let distance = table.get_mod3(index);
            for mv in MOVES {
                let next_index = move_table.get(index, mv) as usize;

                if table.get_mod3(next_index) == 3 {
                    // Not visited yet, set distance and push to queue
                    table.set_mod3(next_index, distance + 1);
                    queue.push_back(next_index);
                }
            }
        }

        table
    }
}

#[derive(Serialize, Deserialize)]
pub struct KociembaTables {
    // Move tables
    pub eo_move: MoveTable,
    pub co_move: MoveTable,
    pub cp_move: MoveTable,
    pub es_move: MoveTable,
    pub ue_move: MoveTable,
    pub de_move: MoveTable,

    // Pruning tables
    pub eo_es_prune: PruningTable,
    pub co_es_prune: PruningTable,
    pub cp_es_prune: PruningTable,
    pub cp_ue_prune: PruningTable,
    pub cp_de_prune: PruningTable,
}

impl KociembaTables {
    pub const DEFAULT_PATH: &'static str = "target/kociemba_tables.bin";

    pub fn load_or_build(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if std::path::Path::new(path).exists() {
            Self::load(path)
        } else {
            let tables = Self::build();
            tables.save(path)?;
            Ok(tables)
        }
    }

    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let tables = bincode::deserialize_from(reader)?;
        Ok(tables)
    }

    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        bincode::serialize_into(writer, self)?;
        Ok(())
    }

    pub fn build() -> Self {
        println!("Building Kociemba tables...");

        // Move tables
        println!("Building move tables...");
        println!("Edge Orientation Move Table...");
        let eo_move = MoveTable::build::<EdgeOrientationIndexer>();
        println!("Corner Orientation Move Table...");
        let co_move = MoveTable::build::<CornerOrientationIndexer>();
        println!("Corner Permutation Move Table...");
        let cp_move = MoveTable::build::<CornerPermutationIndexer>();
        println!("E-Slice Move Table...");
        let es_move = MoveTable::build::<ESliceIndexer>();
        println!("U-Edge Move Table...");
        let ue_move = MoveTable::build::<UEdgeIndexer>();
        println!("D-Edge Move Table...");
        let de_move = MoveTable::build::<DEdgeIndexer>();

        // Pruning tables
        println!("Building pruning tables...");
        println!("EO-ES Pruning Table...");
        let eo_es_prune = PruningTable::build(&eo_move, &es_move);
        println!("CO-ES Pruning Table...");
        let co_es_prune = PruningTable::build(&co_move, &es_move);
        println!("CP-ES Pruning Table...");
        let cp_es_prune = PruningTable::build(&cp_move, &es_move);
        println!("CP-UE Pruning Table...");
        let cp_ue_prune = PruningTable::build(&cp_move, &ue_move);
        println!("CP-DE Pruning Table...");
        let cp_de_prune = PruningTable::build(&cp_move, &de_move);

        Self {
            eo_move,
            co_move,
            cp_move,
            es_move,
            ue_move,
            de_move,
            eo_es_prune,
            co_es_prune,
            cp_es_prune,
            cp_ue_prune,
            cp_de_prune,
        }
    }
}
