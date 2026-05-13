use super::cube::{Cube, Move, Moveable, MOVES};
use super::indexers::*;
use crate::kociemba::phase_solvers::{MOVES_PHASE1, MOVES_PHASE2};
use crate::math::update_distance_mod3;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct IndexMoveTable {
    data: Vec<u16>,
}

impl IndexMoveTable {
    #[inline]
    pub fn get(&self, idx: usize, mv: Move) -> usize {
        self.data[idx * 18 + mv as usize] as usize
    }

    #[inline]
    fn set(&mut self, idx: usize, mv: Move, val: usize) {
        self.data[idx * 18 + mv as usize] = val as u16;
    }

    pub fn build<T: Moveable, I: Indexer<T>>() -> Self {
        assert!(
            I::SIZE <= u16::MAX as usize,
            "Indexer size too large for MoveTable"
        );

        let mut table = Self {
            data: vec![0; I::SIZE * 18],
        };
        for idx1 in 0..I::SIZE {
            for mv in MOVES {
                let mut state = I::from_index(idx1);
                state.turn(mv);
                table.set(idx1, mv, I::to_index(&state));
            }
        }
        table
    }
}

#[derive(Serialize, Deserialize)]
pub struct PruningTable {
    data: Vec<u32>,
    solved_index: usize,
}

impl PruningTable {
    /// Get distance estimate given previous depth estimate
    #[inline]
    pub fn get(&self, index: usize, prev_distance: u32) -> u32 {
        let mod3 = self.get_mod3(index);
        update_distance_mod3(prev_distance, mod3)
    }

    /// Get initial distance lower bound (just the mod-3 value)
    #[inline]
    pub fn get_initial(&self, index: usize) -> u32 {
        self.get_mod3(index)
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

    pub fn build<T: Moveable + Copy, I: Indexer<T>>(solved: T, moves: &[Move]) -> Self {
        let mut table = Self {
            data: vec![0xFFFFFFFF; (I::SIZE >> 4) + 1],
            solved_index: I::to_index(&solved),
        }; // Initialize all entries to 3 (0b11)

        let mut queue = std::collections::VecDeque::new();

        // Starting from solved state.
        table.set_mod3(table.solved_index, 0); // Distance to solved state is 0
        queue.push_back(solved);

        while let Some(state) = queue.pop_front() {
            let distance = table.get_mod3(I::to_index(&state));
            for &mv in moves {
                let mut next = state;
                next.turn(mv);
                let next_index = I::to_index(&next);
                if table.get_mod3(next_index) == 3 {
                    // Not visited yet, set distance and push to queue
                    table.set_mod3(next_index, distance + 1);
                    queue.push_back(next);
                }
            }
        }

        table
    }
}

pub fn compute_min_distance<T: Moveable + Copy, I: Indexer<T>>(
    state: T,
    prune_table: &PruningTable,
    moves: &[Move],
) -> u32 {
    let mut distance = 0;
    let mut current = state;
    let mut current_index = I::to_index(&current);
    let mut current_mod3 = prune_table.get_mod3(current_index);
    while current_index != prune_table.solved_index {
        if current_mod3 == 0 {
            current_mod3 = 3;
        }
        for &mv in moves {
            let mut next = current;
            next.turn(mv);
            let next_index = I::to_index(&next);
            if prune_table.get(next_index, current_mod3) < current_mod3 {
                current = next;
                current_index = next_index;
                current_mod3 = prune_table.get_mod3(current_index);
                distance += 1;
                break;
            }
        }
    }
    distance
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
        let eo_prune = PruningTable::build::<(usize, &IndexMoveTable), EdgeOrientationIndexer>((0, &eo_move), &MOVES_PHASE1);
        println!("CO Pruning Table...");
        let co_prune = PruningTable::build::<(usize, &IndexMoveTable), CornerOrientationIndexer>((0, &co_move), &MOVES_PHASE1);
        println!("CP Pruning Table...");
        let cp_prune = PruningTable::build::<(usize, &IndexMoveTable), CornerPermutationIndexer>((0, &cp_move), &MOVES_PHASE2);
        println!("ES Pruning Table...");
        let es_prune = PruningTable::build::<(usize, &IndexMoveTable), ESliceIndexer>((0, &es_move), &MOVES_PHASE1);
        println!("UE Pruning Table...");
        let ue_prune = PruningTable::build::<(usize, &IndexMoveTable), UEdgeIndexer>((0, &ue_move), &MOVES_PHASE2);
        println!("DE Pruning Table...");
        let de_prune = PruningTable::build::<(usize, &IndexMoveTable), DEdgeIndexer>((0, &de_move), &MOVES_PHASE2);

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







// Impls for simple implementation and testing
impl Moveable for (usize, &IndexMoveTable) {
    fn turn(&mut self, mv: Move) -> &mut Self {
        let (idx, table) = self;
        *idx = table.get(*idx, mv);
        self
    }
}

impl<T: Indexer<Cube>> Indexer<(usize, &IndexMoveTable)> for T {
    const SIZE: usize = T::SIZE;

    fn to_index(state: &(usize, &IndexMoveTable)) -> usize {
        state.0
    }

    fn from_index(_idx: usize) -> (usize, &'static IndexMoveTable) {
        unimplemented!("This is a helper struct for move tables and should not be used directly")
    }
}
