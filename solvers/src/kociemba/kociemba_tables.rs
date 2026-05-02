use super::cube::{Move, MOVES};
use super::indexers::*;
use super::lookup_table::LookupTable2D;

pub type MoveTable = LookupTable2D<u16, 18>;

impl MoveTable {
    #[inline]
    pub fn get(&self, idx: usize, mv: Move) -> u16 {
        self.data[idx * 18 + mv as usize]
    }

    #[inline]
    fn set(&mut self, idx: usize, mv: Move, val: u16) {
        self.data[idx * 18 + mv as usize] = val;
    }

    pub fn build<F: Indexer>() -> Self {
        let mut table = Self::new(F::SIZE, u16::MAX);
        for idx1 in 0..F::SIZE {
            for mv in MOVES {
                let mut cube = F::from_index(idx1);
                cube.turn(mv);
                table.set(idx1, mv, F::to_index(&cube) as u16);
            }
        }
        table
    }
}

pub type PruningTable<const IDX2_WIDTH: usize> = LookupTable2D<u8, IDX2_WIDTH>;

impl<const IDX2_WIDTH: usize> PruningTable<IDX2_WIDTH> {
    #[inline]
    pub fn get(&self, idx1: usize, idx2: usize) -> u8 {
        self.data[idx1 * IDX2_WIDTH + idx2]
        
        // Tightly packed mod 3 distances.
        // let linear_index = idx1 * IDX2_WIDTH + idx2;
        // let word_index = linear_index >> 4;          // divide by 16 (number of entries per u32)
        // let bit_offset = (linear_index & 0xF) << 1;  // (mod 16) * 2
        // (self.data[word_index] >> bit_offset) & 0b11
    }

    #[inline]
    fn set(&mut self, idx1: usize, idx2: usize, val: u8) {
        self.data[idx1 * IDX2_WIDTH + idx2] = val;
    }

    pub fn build(table1: &MoveTable, table2: &MoveTable) -> Self {
        let mut table = Self::new(table1.idx1_width(), u8::MAX);
        let mut queue = std::collections::VecDeque::new();

        // Starting from solved state.
        // Assuming both indexers return 0 for the solved cube
        const SOLVED: (usize, usize) = (0, 0);
        table.set(SOLVED.0, SOLVED.1, 0); // Distance to solved state is 0
        queue.push_back(SOLVED); // (idx1, idx2)

        while let Some((idx1, idx2)) = queue.pop_front() {
            let dist = table.get(idx1, idx2);
            for mv in MOVES {
                let next_idx1 = table1.get(idx1, mv) as usize;
                let next_idx2 = table2.get(idx2, mv) as usize;

                if table.get(next_idx1, next_idx2) == u8::MAX {
                    table.set(next_idx1, next_idx2, dist + 1);
                    queue.push_back((next_idx1, next_idx2));
                }
            }
        }

        table
    }
}

pub struct KociembaTables {
    // Move tables
    pub eo_move: MoveTable,
    pub co_move: MoveTable,
    pub cp_move: MoveTable,
    pub es_move: MoveTable,
    pub ue_move: MoveTable,
    pub de_move: MoveTable,

    // Pruning tables
    pub eo_es_prune: PruningTable<{ ESliceIndexer::SIZE }>,
    pub co_es_prune: PruningTable<{ ESliceIndexer::SIZE }>,
    pub cp_es_prune: PruningTable<{ ESliceIndexer::SIZE }>,
    pub cp_ue_prune: PruningTable<{ UEdgeIndexer::SIZE }>,
    pub cp_de_prune: PruningTable<{ DEdgeIndexer::SIZE }>,
}

impl KociembaTables {
    pub const DEFAULT_PATH: &'static str = "target/kociemba_tables.bin";

    pub fn load_or_build(path: &str) -> Result<Self, std::io::Error> {
        if std::path::Path::new(path).exists() {
            Self::load(path)
        } else {
            let tables = Self::build();
            tables.save(path)?;
            Ok(tables)
        }
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

    pub fn load(file_path: &str) -> Result<Self, std::io::Error> {
        let file = std::fs::File::open(file_path)?;
        let mut reader = std::io::BufReader::new(file);

        // Move tables
        let eo_move = MoveTable::deserialize_from_reader(&mut reader)?;
        let co_move = MoveTable::deserialize_from_reader(&mut reader)?;
        let cp_move = MoveTable::deserialize_from_reader(&mut reader)?;
        let es_move = MoveTable::deserialize_from_reader(&mut reader)?;
        let ue_move = MoveTable::deserialize_from_reader(&mut reader)?;
        let de_move = MoveTable::deserialize_from_reader(&mut reader)?;
        // Pruning tables
        let eo_es_prune = PruningTable::deserialize_from_reader(&mut reader)?;
        let co_es_prune = PruningTable::deserialize_from_reader(&mut reader)?;
        let cp_es_prune = PruningTable::deserialize_from_reader(&mut reader)?;
        let cp_ue_prune = PruningTable::deserialize_from_reader(&mut reader)?;
        let cp_de_prune = PruningTable::deserialize_from_reader(&mut reader)?;

        Ok(Self {
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
        })
    }

    pub fn save(&self, file_path: &str) -> Result<(), std::io::Error> {
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = std::fs::File::create(file_path)?;
        let mut writer = std::io::BufWriter::new(file);

        // Move tables
        self.eo_move.serialize_to_writer(&mut writer)?;
        self.co_move.serialize_to_writer(&mut writer)?;
        self.cp_move.serialize_to_writer(&mut writer)?;
        self.es_move.serialize_to_writer(&mut writer)?;
        self.ue_move.serialize_to_writer(&mut writer)?;
        self.de_move.serialize_to_writer(&mut writer)?;
        // Pruning tables
        self.eo_es_prune.serialize_to_writer(&mut writer)?;
        self.co_es_prune.serialize_to_writer(&mut writer)?;
        self.cp_es_prune.serialize_to_writer(&mut writer)?;
        self.cp_ue_prune.serialize_to_writer(&mut writer)?;
        self.cp_de_prune.serialize_to_writer(&mut writer)?;

        Ok(())
    }
}
