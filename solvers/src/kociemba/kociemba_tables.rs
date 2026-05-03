use super::cube::{Cube, Move, MOVES, Moveable};
use super::indexers::*;
use crate::math::update_distance_mod3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct IndexMoveTable {
    data: Vec<u16>,
}

impl IndexMoveTable {
    #[inline]
    pub fn get(&self, idx: usize, mv: Move) -> u16 {
        self.data[idx * 18 + mv as usize]
    }

    #[inline]
    fn set(&mut self, idx: usize, mv: Move, val: u16) {
        self.data[idx * 18 + mv as usize] = val;
    }

    pub fn build<T: Moveable, I: Indexer<T>>() -> Self {
        assert!(I::SIZE <= u16::MAX as usize, "Indexer size too large for MoveTable");
        
        let mut table = Self {
            data: vec![0; I::SIZE * 18],
        };
        for idx1 in 0..I::SIZE {
            for mv in MOVES {
                let mut state = I::from_index(idx1);
                state.turn(mv);
                table.set(idx1, mv, I::to_index(&state) as u16);
            }
        }
        table
    }
}

#[derive(Serialize, Deserialize)]
pub struct PruningTable {
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

    pub fn build(move_table: &IndexMoveTable, solved_index: usize) -> Self {
        let width = move_table.data.len() / 18;
        let mut table = Self {
            data: vec![0xFFFFFFFF; width >> 4 + 1],
        }; // Initialize all entries to 3 (0b11)
        
        let mut queue = std::collections::VecDeque::new();

        // Starting from solved state.
        table.set_mod3(solved_index, 0); // Distance to solved state is 0
        queue.push_back(solved_index);

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
    pub eo_move: IndexMoveTable,
    pub co_move: IndexMoveTable,
    pub cp_move: IndexMoveTable,
    pub es_move: IndexMoveTable,
    pub ue_move: IndexMoveTable,
    pub de_move: IndexMoveTable,

    // Pruning tables
    pub eo_prune: PruningTable,
    pub co_prune: PruningTable,
    pub cp_prune: PruningTable,
    pub es_prune: PruningTable,
    pub ue_prune: PruningTable,
    pub de_prune: PruningTable,
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
        let eo_move = IndexMoveTable::build::<Cube, EdgeOrientationIndexer>();
        println!("Corner Orientation Move Table...");
        let co_move = IndexMoveTable::build::<Cube, CornerOrientationIndexer>();
        println!("Corner Permutation Move Table...");
        let cp_move = IndexMoveTable::build::<Cube, CornerPermutationIndexer>();
        println!("E-Slice Move Table...");
        let es_move = IndexMoveTable::build::<Cube, ESliceIndexer>();
        println!("U-Edge Move Table...");
        let ue_move = IndexMoveTable::build::<Cube, UEdgeIndexer>();
        println!("D-Edge Move Table...");
        let de_move = IndexMoveTable::build::<Cube, DEdgeIndexer>();

        // Pruning tables
        println!("Building pruning tables...");
        println!("EO Pruning Table...");
        let eo_prune = PruningTable::build(&eo_move, EdgeOrientationIndexer::SOLVED_INDEX);
        println!("CO Pruning Table...");
        let co_prune = PruningTable::build(&co_move, CornerOrientationIndexer::SOLVED_INDEX);
        println!("CP Pruning Table...");
        let cp_prune = PruningTable::build(&cp_move, CornerPermutationIndexer::SOLVED_INDEX);
        println!("ES Pruning Table...");
        let es_prune = PruningTable::build(&es_move, ESliceIndexer::SOLVED_INDEX);
        println!("UE Pruning Table...");
        let ue_prune = PruningTable::build(&ue_move, UEdgeIndexer::SOLVED_INDEX);
        println!("DE Pruning Table...");
        let de_prune = PruningTable::build(&de_move, DEdgeIndexer::SOLVED_INDEX);

        Self {
            eo_move,
            co_move,
            cp_move,
            es_move,
            ue_move,
            de_move,
            eo_prune,
            co_prune,
            cp_prune,
            es_prune,
            ue_prune,
            de_prune,
        }
    }
}
