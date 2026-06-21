use super::indexers::*;
use super::phase_states::{
    EOSIndexer, Phase1Indexer, Phase2Indexer1, Phase2Indexer2, CP_SYMMETRY_CLASSES,
    EOS_SYMMETRY_CLASSES, MOVES_PHASE1, MOVES_PHASE2,
};
use cube::math::update_distance_mod3;
use cube::symmetries::D4h_SYMMETRIES;
use cube::symmetries::{INV_INDEX_MAP, SYMMETRIES};
use cube::{Cube, Move, Moveable, MOVES};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

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

    pub fn build<T: Moveable, I: Indexer<T>>(indexer: I) -> Self {
        assert!(
            I::SIZE <= u16::MAX as usize,
            "Indexer size too large for MoveTable"
        );

        let mut table = Self {
            data: vec![0; I::SIZE * 18],
        };
        for idx1 in 0..I::SIZE {
            for mv in MOVES {
                let mut state = indexer.from_index(idx1);
                state.turn(mv);
                table.set(idx1, mv, indexer.to_index(&state));
            }
        }
        table
    }
}

#[derive(Serialize, Deserialize)]
pub struct SymmetryReductionTable {
    symmetry_class_map: Vec<(u16, u8)>,
    representatives_map: Vec<u32>,
}

impl SymmetryReductionTable {
    /// Given `index`, returns the index of its equivalence class and
    /// the symmetry that maps `index` to the class representative.
    pub fn get_class(&self, index: usize) -> (usize, usize) {
        let (class_index, sym_index) = self.symmetry_class_map[index];
        (class_index as usize, sym_index as usize)
    }

    pub fn get_representative(&self, class_index: usize) -> usize {
        self.representatives_map[class_index] as usize
    }

    pub fn build<I: Indexer<Cube>>(indexer: I, symmetries: &[usize]) -> Self {
        let mut symmetry_class_map = vec![(u16::MAX, u8::MAX); I::SIZE];
        let mut representatives_map = Vec::new();
        let mut class_index = 0;

        for base_index in 0..I::SIZE {
            assert!(
                class_index < u16::MAX as usize,
                "Too many symmetry classes to store as u16 (max {}, got {})",
                u16::MAX,
                class_index
            );
            assert!(
                base_index < u32::MAX as usize,
                "Not possible to store representatives as u32 (max {}, got {})",
                u32::MAX,
                base_index
            );

            if symmetry_class_map[base_index] != (u16::MAX, u8::MAX) {
                // Already assigned to a class, skip
                continue;
            }

            let base_cube = indexer.from_index(base_index);

            representatives_map.push(base_index as u32);

            for &sym_index in symmetries {
                let sym = SYMMETRIES[sym_index];
                let cube = sym * base_cube;
                let index = indexer.to_index(&cube);
                if symmetry_class_map[index] == (u16::MAX, u8::MAX) {
                    // This check is technically not needed, but it does guarantee that the
                    // symmetry stored with the class representatives is the identity.
                    let inv_sym_index = INV_INDEX_MAP[sym_index];
                    symmetry_class_map[index] = (class_index as u16, inv_sym_index as u8);
                }
            }

            class_index += 1;
        }

        println!(
            "    Reduction complete: {} unique classes over {} states (compression: {:.1}x, {:.2}% of original)",
            class_index,
            I::SIZE,
            I::SIZE as f64 / class_index as f64,
            class_index as f64 * 100.0 / I::SIZE as f64
        );
        Self {
            symmetry_class_map,
            representatives_map,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SymmetryConjugationTable {
    data: Vec<u16>,
    local_index: Vec<Option<u8>>,
    num_symmetries: usize,
}

impl SymmetryConjugationTable {
    #[inline]
    pub fn get(&self, index: usize, mut sym_index: usize) -> usize {
        sym_index = self.local_index[sym_index].expect("Symmetry not in this table") as usize;
        self.data[index * self.num_symmetries + sym_index] as usize
    }

    pub fn build<I: Indexer<Cube>>(indexer: I, symmetries: &[usize]) -> Self {
        assert!(
            I::SIZE <= u16::MAX as usize,
            "Indexer too large to store as u16 (max {}, got {})",
            u16::MAX,
            I::SIZE
        );

        let mut data = vec![0; I::SIZE * symmetries.len()];

        let mut local_index = vec![None; 48];
        for (local_idx, &global_idx) in symmetries.iter().enumerate() {
            local_index[global_idx] = Some(local_idx as u8);
        }

        // Build conjugation table
        for coord in 0..I::SIZE {
            let cube = indexer.from_index(coord);
            for (local_idx, &global_idx) in symmetries.iter().enumerate() {
                let sym = SYMMETRIES[global_idx];
                let conjugated = sym * cube;
                data[coord * symmetries.len() + local_idx] = indexer.to_index(&conjugated) as u16;
            }
        }

        Self {
            data,
            num_symmetries: symmetries.len(),
            local_index,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PruningTable {
    data: Vec<u32>,
}

impl PruningTable {
    /// Get distance estimate given previous depth estimate
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

    pub fn build<T: Moveable + Copy, I: Indexer<T>>(indexer: I, moves: &[Move]) -> Self {
        let mut table = Self {
            data: vec![0xFFFFFFFF; (I::SIZE >> 4) + 1],
        }; // Initialize all entries to 3 (0b11)

        table.set_mod3(I::SOLVED_INDEX, 0);

        let mut depth = 1;
        let mut mod3 = 1;
        let mut prev_mod3 = 0;
        let mut visited = 1;
        while visited < I::SIZE {
            let mut idx = 0;
            while idx < I::SIZE {
                if idx % (I::SIZE / 10) == 0 || idx == I::SIZE {
                    print!(
                        "\r      Depth {:>2}: {:>3}% | Total: {:>5.1}% ({}/{})",
                        depth,
                        idx * 100 / I::SIZE,
                        (visited * 1000 / I::SIZE) as f64 / 10.0,
                        visited,
                        I::SIZE
                    );
                    io::stdout().flush().unwrap();
                }
                
                if table.data[idx >> 4] == 0xFFFFFFFF {
                    idx += 16; // Skip 16 entries at once if all are unvisited
                    continue;
                }
                if table.get_mod3(idx) != prev_mod3 {
                    idx += 1;
                    continue;
                }

                // State was reached at previous depth. Update neighbours.
                let state = indexer.from_index(idx);
                for &mv in moves {
                    let mut next = state;
                    next.turn(mv);
                    let next_idx = indexer.to_index(&next);
                    if table.get_mod3(next_idx) == 3 {
                        table.set_mod3(next_idx, mod3);
                        visited += 1;
                    }
                }

                idx += 1;
            }


            depth += 1;
            prev_mod3 = mod3;
            mod3 = depth % 3;
        }
        
        table
    }
}

pub fn compute_min_distance<T: Moveable + Copy, I: Indexer<T>>(
    mut state: T,
    indexer: &I,
    prune_table: &PruningTable,
    moves: &[Move],
) -> u32 {
    let mut distance = 0;
    let mut index = indexer.to_index(&state);
    let mut reference_dist = 3 + prune_table.get_mod3(index);
    while index != I::SOLVED_INDEX {
        for &mv in moves {
            state.turn(mv);
            let next_index = indexer.to_index(&state);
            let next_mod3 = prune_table.get_mod3(next_index);
            if update_distance_mod3(reference_dist, next_mod3) < reference_dist {
                index = next_index;
                reference_dist = 3 + next_mod3;
                distance += 1;
                break;
            }
            state.turn(mv.inverse());
        }
    }
    distance
}

#[derive(Serialize, Deserialize)]
pub struct MoveTables {
    pub eo_move: IndexMoveTable,
    pub co_move: IndexMoveTable,
    pub cp_move: IndexMoveTable,
    pub esc_move: IndexMoveTable,
    pub ue_move: IndexMoveTable,
    pub de_move: IndexMoveTable,
    pub ud_move: IndexMoveTable,
    pub esp_move: IndexMoveTable, // we need two for e slice edges!
}

impl MoveTables {
    pub fn build() -> Self {
        println!("Building move tables...");
        println!("  Building EO move table...");
        let eo_move = IndexMoveTable::build::<Cube, EdgeOrientationIndexer>(EdgeOrientationIndexer);
        println!("  Building CO move table...");
        let co_move =
            IndexMoveTable::build::<Cube, CornerOrientationIndexer>(CornerOrientationIndexer);
        println!("  Building CP move table...");
        let cp_move =
            IndexMoveTable::build::<Cube, CornerPermutationIndexer>(CornerPermutationIndexer);
        println!("  Building ESC move table...");
        let esc_move =
            IndexMoveTable::build::<Cube, ESliceCombinationIndexer>(ESliceCombinationIndexer);
        println!("  Building UE move table...");
        let ue_move = IndexMoveTable::build::<Cube, UEdgeIndexer>(UEdgeIndexer);
        println!("  Building DE move table...");
        let de_move = IndexMoveTable::build::<Cube, DEdgeIndexer>(DEdgeIndexer);
        println!("  Building UD move table...");
        let ud_move =
            IndexMoveTable::build::<Cube, UDEdgePermutationIndexer>(UDEdgePermutationIndexer);
        println!("  Building ESP move table...");
        let esp_move =
            IndexMoveTable::build::<Cube, ESlicePermutationIndexer>(ESlicePermutationIndexer);
        println!("Move tables complete!");

        Self {
            eo_move,
            co_move,
            cp_move,
            esc_move,
            ue_move,
            de_move,
            ud_move,
            esp_move,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SymmetryTables {
    pub eos_reduction: SymmetryReductionTable,
    pub co_conjugation: SymmetryConjugationTable,
    pub cp_reduction: SymmetryReductionTable,
    pub ud_conjugation: SymmetryConjugationTable,
}

impl SymmetryTables {
    pub fn build() -> Self {
        println!("Building symmetry tables...");
        println!("  Building EOS reduction table...");
        let eos_reduction = SymmetryReductionTable::build(EOSIndexer, &D4h_SYMMETRIES);
        println!("  Building CO conjugation table...");
        let co_conjugation =
            SymmetryConjugationTable::build(CornerOrientationIndexer, &D4h_SYMMETRIES);
        println!("  Building CP reduction table...");
        let cp_reduction = SymmetryReductionTable::build(CornerPermutationIndexer, &D4h_SYMMETRIES);
        println!("  Building UD conjugation table...");
        let ud_conjugation =
            SymmetryConjugationTable::build(UDEdgePermutationIndexer, &D4h_SYMMETRIES);
        println!("Symmetry tables complete!");
        Self {
            eos_reduction,
            co_conjugation,
            cp_reduction,
            ud_conjugation,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PruningTables {
    pub phase1_prune: PruningTable,
    pub phase2_prune1: PruningTable,
    pub phase2_prune2: PruningTable,
}

impl PruningTables {
    pub fn build(move_tables: &MoveTables, sym_tables: &SymmetryTables) -> Self {
        println!("Building pruning tables...");
        let phase1_indexer = Phase1Indexer::new(move_tables, sym_tables);
        let phase2_indexer1 = Phase2Indexer1::new(move_tables, sym_tables);
        let phase2_indexer2 = Phase2Indexer2::new(move_tables);

        assert_eq!(
            sym_tables.eos_reduction.representatives_map.len(),
            EOS_SYMMETRY_CLASSES,
            "EOS reduction table has wrong number of classes"
        );
        assert_eq!(
            sym_tables.cp_reduction.representatives_map.len(),
            CP_SYMMETRY_CLASSES,
            "CP reduction table has wrong number of classes"
        );

        println!("  Building Phase 1 pruning table (this may take a while)...");
        let phase1_prune = PruningTable::build(phase1_indexer, &MOVES_PHASE1);
        println!("  Building Phase 2 pruning table 1 (CP*UD)...");
        let phase2_prune1 = PruningTable::build(phase2_indexer1, &MOVES_PHASE2);
        println!("  Building Phase 2 pruning table 2 (CP*ESP)...");
        let phase2_prune2 = PruningTable::build(phase2_indexer2, &MOVES_PHASE2);
        println!("Pruning tables complete!");

        Self {
            phase1_prune,
            phase2_prune1,
            phase2_prune2,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct KociembaTables {
    pub move_tables: MoveTables,
    pub symmetry_tables: SymmetryTables,
    pub pruning_tables: PruningTables,
}

impl KociembaTables {
    pub const DEFAULT_PATH: &'static str = "target/kociemba_tables.bin";

    pub fn build() -> Self {
        println!("\n=== Building Kociemba Tables ===");
        let start = std::time::Instant::now();

        let move_tables = MoveTables::build();
        let symmetry_tables = SymmetryTables::build();
        let pruning_tables = PruningTables::build(&move_tables, &symmetry_tables);

        println!(
            "\nAll tables built in {:.2}s\n",
            start.elapsed().as_secs_f64()
        );

        Self {
            move_tables,
            symmetry_tables,
            pruning_tables,
        }
    }

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
        println!("Saving tables to {}...", path);
        if let Some(parent) = std::path::Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = std::fs::File::create(path)?;
        let writer = std::io::BufWriter::new(file);
        bincode::serialize_into(writer, self)?;
        println!("Tables saved successfully!");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    static MOVE_TABLES: LazyLock<MoveTables> = LazyLock::new(|| MoveTables::build());
    static SYMMETRY_TABLES: LazyLock<SymmetryTables> = LazyLock::new(|| SymmetryTables::build());

    macro_rules! test_movetable {
        ($mod_name:ident, $indexer:path, $movepool:expr) => {
            mod $mod_name {
                use super::*;

                #[test]
                fn consistency() {
                    let indexer = $indexer;
                    let table = &MOVE_TABLES.$mod_name;
                    for idx1 in 0..<$indexer as Indexer<Cube>>::SIZE {
                        for mv in $movepool {
                            let mut state = indexer.from_index(idx1);
                            state.turn(mv);
                            let idx2_table = table.get(idx1, mv);
                            let idx2_direct = indexer.to_index(&state);
                            assert_eq!(
                                idx2_table, idx2_direct,
                                "Move table failed for move {:?} at index {}",
                                mv, idx1
                            );
                            let idx1_back = table.get(idx2_table, mv.inverse());
                            assert_eq!(
                                idx1_back, idx1,
                                "Move table reversibility failed for move {:?} at index {}",
                                mv, idx1
                            );
                        }
                    }
                }
            }
        };
    }

    test_movetable!(eo_move, EdgeOrientationIndexer, MOVES);
    test_movetable!(co_move, CornerOrientationIndexer, MOVES);
    test_movetable!(cp_move, CornerPermutationIndexer, MOVES);
    test_movetable!(esc_move, ESliceCombinationIndexer, MOVES);
    test_movetable!(ue_move, UEdgeIndexer, MOVES);
    test_movetable!(de_move, DEdgeIndexer, MOVES);
    test_movetable!(ud_move, UDEdgePermutationIndexer, MOVES_PHASE2);
    test_movetable!(esp_move, ESlicePermutationIndexer, MOVES_PHASE2);

    macro_rules! test_symmetry_reduction {
        ($mod_name:ident, $indexer:path, $symmetries:expr) => {
            mod $mod_name {
                use super::*;

                #[test]
                fn consistency() {
                    let indexer = $indexer;
                    let table = &SYMMETRY_TABLES.$mod_name;
                    for idx in 0..<$indexer as Indexer<Cube>>::SIZE {
                        let (class_idx, sym_idx) = table.get_class(idx);
                        let representative = table.get_representative(class_idx);

                        let cube: Cube = indexer.from_index(idx);
                        let sym = SYMMETRIES[sym_idx];
                        let reduced_cube = sym * cube;
                        let reduced = indexer.to_index(&reduced_cube);
                        assert_eq!(
                            reduced, representative,
                            "Index {idx} was not reduced to its representative by the given symmetry",
                        );
                    }
                }

                #[test]
                fn symmetry_invariance() {
                    let indexer = $indexer;
                    let table = &SYMMETRY_TABLES.$mod_name;
                    for idx in 0..<$indexer as Indexer<Cube>>::SIZE {
                        let cube: Cube = indexer.from_index(idx);
                        let (class, _) = table.get_class(idx);
                        for &sym_index in $symmetries {
                            let sym = SYMMETRIES[sym_index];
                            let sym_cube = sym * cube;
                            let sym_idx = indexer.to_index(&sym_cube);
                            let (class2, _) = table.get_class(sym_idx);
                            assert_eq!(
                                class, class2,
                                "Symmetry reduction should be invariant for symmetry index {}",
                                sym_index
                            );
                        }
                    }
                }
            }
        };
    }

    test_symmetry_reduction!(eos_reduction, EOSIndexer, &D4h_SYMMETRIES);
    test_symmetry_reduction!(cp_reduction, CornerPermutationIndexer, &D4h_SYMMETRIES);
}
