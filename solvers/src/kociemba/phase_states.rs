use super::indexers::*;
use super::tables::{
    IndexMoveTable, MoveTables, SymmetryConjugationTable, SymmetryReductionTable, SymmetryTables,
};
use cube::symmetries::{D4h_SYMMETRIES, Symmetry, INV_INDEX_MAP};
use cube::{Cube, Move, Moveable, EDGES, MOVES};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

pub const EOS_SYMMETRY_CLASSES: usize = 64430;
pub const CP_SYMMETRY_CLASSES: usize = 2768;

pub const MOVES_PHASE1: [Move; 18] = MOVES;
pub const MOVES_PHASE2: [Move; 10] = [
    Move::R2,
    Move::L2,
    Move::U,
    Move::Up,
    Move::U2,
    Move::D,
    Move::Dp,
    Move::D2,
    Move::F2,
    Move::B2,
];

#[derive(Clone, Copy)]
pub struct Phase1State<'a> {
    pub eo: usize,
    pub co: usize,
    pub esc: usize,

    pub eo_move_table: &'a IndexMoveTable,
    pub co_move_table: &'a IndexMoveTable,
    pub esc_move_table: &'a IndexMoveTable,
}

impl Moveable for Phase1State<'_> {
    fn turn(&mut self, mv: Move) -> &mut Self {
        self.eo = self.eo_move_table.get(self.eo, mv);
        self.co = self.co_move_table.get(self.co, mv);
        self.esc = self.esc_move_table.get(self.esc, mv);
        self
    }
}

impl<'a> Phase1State<'a> {
    pub fn is_solved(&self) -> bool {
        self.eo == EdgeOrientationIndexer::SOLVED_INDEX
            && self.co == CornerOrientationIndexer::SOLVED_INDEX
            && self.esc == ESliceCombinationIndexer::SOLVED_INDEX
    }

    pub fn from_cube(cube: &Cube, tables: &'a MoveTables) -> Self {
        let eo = EdgeOrientationIndexer.to_index(cube);
        let co = CornerOrientationIndexer.to_index(cube);
        let esc = ESliceCombinationIndexer.to_index(cube);
        let eo_move_table = &tables.eo_move;
        let co_move_table = &tables.co_move;
        let esc_move_table = &tables.esc_move;

        Self {
            eo,
            co,
            esc,
            eo_move_table,
            co_move_table,
            esc_move_table,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Phase1Indexer<'a> {
    pub eos_reduction_table: &'a SymmetryReductionTable,
    pub co_symmetry_table: &'a SymmetryConjugationTable,
    pub move_tables: &'a MoveTables,
}

impl<'a> Phase1Indexer<'a> {
    pub fn new(move_tables: &'a MoveTables, sym_tables: &'a SymmetryTables) -> Self {
        Self {
            eos_reduction_table: &sym_tables.eos_reduction,
            co_symmetry_table: &sym_tables.co_conjugation,
            move_tables,
        }
    }
}

impl<'a> Indexer<Phase1State<'a>> for Phase1Indexer<'a> {
    const SIZE: usize = EOS_SYMMETRY_CLASSES * CornerOrientationIndexer::SIZE;
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, state: &Phase1State<'a>) -> usize {
        let eos = EOSIndexer.to_index(&(state.eo, state.esc));
        let (eos_class, sym) = self.eos_reduction_table.get_class(eos);
        let co_conj = self.co_symmetry_table.get(state.co, sym);
        eos_class * CornerOrientationIndexer::SIZE + co_conj
    }

    fn from_index(&self, index: usize) -> Phase1State<'a> {
        let eos_class = index / CornerOrientationIndexer::SIZE;
        let co_conj = index % CornerOrientationIndexer::SIZE;

        let eos = self.eos_reduction_table.get_representative(eos_class);
        let (eo, esc) = EOSIndexer.from_index(eos);

        let (_, sym) = self.eos_reduction_table.get_class(eos);
        let inv_sym = INV_INDEX_MAP[sym];
        let co = self.co_symmetry_table.get(co_conj, inv_sym);

        // co = co_conj because get_class on a representative returns the identity symmetry
        Phase1State {
            eo,
            co,
            esc,
            eo_move_table: &self.move_tables.eo_move,
            co_move_table: &self.move_tables.co_move,
            esc_move_table: &self.move_tables.esc_move,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Phase2State<'a> {
    pub cp: usize,
    pub ud: usize,
    pub esp: usize,

    pub cp_move_table: &'a IndexMoveTable,
    pub ud_move_table: &'a IndexMoveTable,
    pub esp_move_table: &'a IndexMoveTable,
}

impl<'a> Moveable for Phase2State<'_> {
    fn turn(&mut self, mv: Move) -> &mut Self {
        self.cp = self.cp_move_table.get(self.cp, mv);
        self.ud = self.ud_move_table.get(self.ud, mv);
        self.esp = self.esp_move_table.get(self.esp, mv);
        self
    }
}

impl<'a> Phase2State<'a> {
    pub fn is_solved(&self) -> bool {
        self.cp == CornerPermutationIndexer::SOLVED_INDEX
            && self.ud == UDEdgePermutationIndexer::SOLVED_INDEX
            && self.esp == ESliceIndexer::SOLVED_INDEX
    }

    pub fn from_cube(cube: &Cube, tables: &'a MoveTables) -> Self {
        let cp = CornerPermutationIndexer.to_index(cube);
        let ud = UDEdgePermutationIndexer.to_index(cube);
        let esp = ESliceIndexer.to_index(cube);
        let cp_move_table = &tables.cp_move;
        let ud_move_table = &tables.ud_move;
        let esp_move_table = &tables.esp_move;

        Self {
            cp,
            ud,
            esp,
            cp_move_table,
            ud_move_table,
            esp_move_table,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Phase2Indexer1<'a> {
    pub cp_reduction_table: &'a SymmetryReductionTable,
    pub ud_symmetry_table: &'a SymmetryConjugationTable,
    pub move_tables: &'a MoveTables,
}

impl<'a> Phase2Indexer1<'a> {
    pub fn new(move_tables: &'a MoveTables, sym_tables: &'a SymmetryTables) -> Self {
        Self {
            cp_reduction_table: &sym_tables.cp_reduction,
            ud_symmetry_table: &sym_tables.ud_conjugation,
            move_tables,
        }
    }
}

impl<'a> Indexer<Phase2State<'a>> for Phase2Indexer1<'a> {
    const SIZE: usize = CP_SYMMETRY_CLASSES * UDEdgePermutationIndexer::SIZE;
    const SOLVED_INDEX: usize = 0; // TODO: Calculate actual solved index

    fn to_index(&self, state: &Phase2State<'a>) -> usize {
        let (cp_class, sym) = self.cp_reduction_table.get_class(state.cp);
        let ud_conj = self.ud_symmetry_table.get(state.ud, sym);
        cp_class * UDEdgePermutationIndexer::SIZE + ud_conj
    }

    fn from_index(&self, index: usize) -> Phase2State<'a> {
        let cp_class = index / UDEdgePermutationIndexer::SIZE;
        let ud_conj = index % UDEdgePermutationIndexer::SIZE;
        let cp = self.cp_reduction_table.get_representative(cp_class);

        let (_, sym) = self.cp_reduction_table.get_class(cp);
        let inv_sym = INV_INDEX_MAP[sym];
        let ud = self.ud_symmetry_table.get(ud_conj, inv_sym);

        // ud = ud_conj because get_class on a representative returns the identity symmetry
        Phase2State {
            cp,
            ud,
            esp: ESliceIndexer::SOLVED_INDEX,
            cp_move_table: &self.move_tables.cp_move,
            ud_move_table: &self.move_tables.ud_move,
            esp_move_table: &self.move_tables.esp_move,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Phase2Indexer2<'a> {
    pub move_tables: &'a MoveTables,
}

impl<'a> Phase2Indexer2<'a> {
    pub fn new(move_tables: &'a MoveTables) -> Self {
        Self { move_tables }
    }
}

impl<'a> Indexer<Phase2State<'a>> for Phase2Indexer2<'a> {
    const SIZE: usize = CornerPermutationIndexer::SIZE * ESlicePermutationIndexer::SIZE;
    const SOLVED_INDEX: usize = 0; // TODO: Calculate actual solved index

    fn to_index(&self, state: &Phase2State<'a>) -> usize {
        state.cp * ESlicePermutationIndexer::SIZE + state.esp
    }

    fn from_index(&self, index: usize) -> Phase2State<'a> {
        let cp = index / ESlicePermutationIndexer::SIZE;
        let esp = index % ESlicePermutationIndexer::SIZE;
        Phase2State {
            cp,
            ud: UDEdgePermutationIndexer::SOLVED_INDEX,
            esp,
            cp_move_table: &self.move_tables.cp_move,
            ud_move_table: &self.move_tables.ud_move,
            esp_move_table: &self.move_tables.esp_move,
        }
    }
}

pub struct EOSIndexer;

impl Indexer<Cube> for EOSIndexer {
    const SIZE: usize = EdgeOrientationIndexer::SIZE * ESliceCombinationIndexer::SIZE;
    const SOLVED_INDEX: usize = EdgeOrientationIndexer::SOLVED_INDEX
        * ESliceCombinationIndexer::SIZE
        + ESliceCombinationIndexer::SOLVED_INDEX;

    fn to_index(&self, cube: &Cube) -> usize {
        let eo_indexer = EdgeOrientationIndexer;
        let esc_indexer = ESliceCombinationIndexer;
        let eo_idx = eo_indexer.to_index(cube);
        let esc_idx = esc_indexer.to_index(cube);
        eo_idx * ESliceCombinationIndexer::SIZE + esc_idx
    }

    fn from_index(&self, index: usize) -> Cube {
        let (eo, esc) = self.from_index(index);
        let orientation_cube = EdgeOrientationIndexer.from_index(eo);
        let mut cube = ESliceCombinationIndexer.from_index(esc);
        for &position in EDGES.iter() {
            let orientation = orientation_cube.get_edge_orientation(position);
            cube.set_edge_orientation(position, orientation);
        }
        cube
    }
}

impl Indexer<(usize, usize)> for EOSIndexer {
    const SIZE: usize = EdgeOrientationIndexer::SIZE * ESliceCombinationIndexer::SIZE;
    const SOLVED_INDEX: usize = EdgeOrientationIndexer::SOLVED_INDEX
        * ESliceCombinationIndexer::SIZE
        + ESliceCombinationIndexer::SOLVED_INDEX;

    fn to_index(&self, state: &(usize, usize)) -> usize {
        let eo_idx = state.0;
        let esc_idx = state.1;
        eo_idx * ESliceCombinationIndexer::SIZE + esc_idx
    }

    fn from_index(&self, index: usize) -> (usize, usize) {
        let eo_idx = index / ESliceCombinationIndexer::SIZE;
        let esc_idx = index % ESliceCombinationIndexer::SIZE;
        (eo_idx, esc_idx)
    }
}

pub trait SymmetryReducedIndexer<T>: Indexer<T> {
    fn equivalent_indices(&self, index: usize) -> impl Iterator<Item = usize>;
}

static EOS_STABILIZER_CACHE: LazyLock<Mutex<HashMap<usize, u16>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl<'a> SymmetryReducedIndexer<Phase1State<'a>> for Phase1Indexer<'a> {
    fn equivalent_indices(&self, index: usize) -> impl Iterator<Item = usize> {
        let co = index % CornerOrientationIndexer::SIZE;
        let eos_class = index / CornerOrientationIndexer::SIZE;

        let stabiliser_mask = *EOS_STABILIZER_CACHE
            .lock()
            .unwrap()
            .entry(eos_class)
            .or_insert_with(|| {
                let eos = self.eos_reduction_table.get_representative(eos_class);
                let mut mask = 0;
                for sym in D4h_SYMMETRIES {
                    let mut other = <EOSIndexer as Indexer<Cube>>::from_index(&EOSIndexer, eos);
                    other = Symmetry::cube_conjugation(&other, sym);
                    let eos2 = EOSIndexer.to_index(&other);
                    if eos2 == eos {
                        mask |= 1 << sym;
                    }
                }
                mask
            });

        let mut indices = Vec::with_capacity(D4h_SYMMETRIES.len());
        for sym in (0..D4h_SYMMETRIES.len()).filter(|&s| stabiliser_mask & (1 << s) != 0) {
            let co2 = self.co_symmetry_table.get(co, sym);
            let index2 = eos_class * CornerOrientationIndexer::SIZE + co2;
            if !indices.contains(&index2) {
                indices.push(index2);
            }
        }
        indices.into_iter()
    }
}

static CP_STABILIZER_CACHE: LazyLock<Mutex<HashMap<usize, u16>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

impl<'a> SymmetryReducedIndexer<Phase2State<'a>> for Phase2Indexer1<'a> {
    fn equivalent_indices(&self, index: usize) -> impl Iterator<Item = usize> {
        let phase2state = self.from_index(index);
        let ud = phase2state.ud;
        let cp = phase2state.cp;
        let cp_class = self.cp_reduction_table.get_class(cp).0;

        let stabiliser_mask = *CP_STABILIZER_CACHE
            .lock()
            .unwrap()
            .entry(cp_class)
            .or_insert_with(|| {
                let cp = self.cp_reduction_table.get_representative(cp_class);
                let mut mask = 0;
                for sym in D4h_SYMMETRIES {
                    let mut other = <CornerPermutationIndexer as Indexer<Cube>>::from_index(
                        &CornerPermutationIndexer,
                        cp,
                    );
                    other = Symmetry::cube_conjugation(&other, sym);
                    let cp2 = CornerPermutationIndexer.to_index(&other);
                    if cp2 == cp {
                        mask |= 1 << sym;
                    }
                }
                mask
            });

        let mut indices = Vec::with_capacity(D4h_SYMMETRIES.len());
        for sym in (0..D4h_SYMMETRIES.len()).filter(|&s| stabiliser_mask & (1 << s) != 0) {
            let ud2 = self.ud_symmetry_table.get(ud, sym);
            let index2 = cp_class * UDEdgePermutationIndexer::SIZE + ud2;
            if !indices.contains(&index2) {
                indices.push(index2);
            }
        }
        indices.into_iter()
    }
}

impl<'a> SymmetryReducedIndexer<Phase2State<'a>> for Phase2Indexer2<'a> {
    fn equivalent_indices(&self, _index: usize) -> impl Iterator<Item = usize> {
        Vec::new().into_iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_indexer;

    use super::*;
    use std::sync::LazyLock;

    static MOVE_TABLES: LazyLock<MoveTables> = LazyLock::new(|| MoveTables::build());
    static SYMMETRY_TABLES: LazyLock<SymmetryTables> = LazyLock::new(|| SymmetryTables::build());
    static P1I: LazyLock<Phase1Indexer> =
        LazyLock::new(|| Phase1Indexer::new(&MOVE_TABLES, &SYMMETRY_TABLES));
    static P2I1: LazyLock<Phase2Indexer1> =
        LazyLock::new(|| Phase2Indexer1::new(&MOVE_TABLES, &SYMMETRY_TABLES));
    static P2I2: LazyLock<Phase2Indexer2> = LazyLock::new(|| Phase2Indexer2::new(&MOVE_TABLES));

    test_indexer!(eos, EOSIndexer, EOSIndexer, Cube, Cube::new_solved());
    test_indexer!(
        phase1,
        Phase1Indexer,
        &P1I,
        Phase1State,
        Phase1State::from_cube(&Cube::new_solved(), &MOVE_TABLES)
    );
    test_indexer!(
        phase2_1,
        Phase2Indexer1,
        &P2I1,
        Phase2State,
        Phase2State::from_cube(&Cube::new_solved(), &MOVE_TABLES)
    );
    test_indexer!(
        phase2_2,
        Phase2Indexer2,
        &P2I2,
        Phase2State,
        Phase2State::from_cube(&Cube::new_solved(), &MOVE_TABLES)
    );
}
