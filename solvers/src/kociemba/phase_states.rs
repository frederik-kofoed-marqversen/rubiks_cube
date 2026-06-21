use crate::kociemba::tables::MoveTables;

use super::indexers::*;
use super::tables::{IndexMoveTable, SymmetryConjugationTable, SymmetryReductionTable};
use super::KociembaTables;
use cube::{Cube, Move, Moveable, MOVES, EDGES};

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
}

impl<'a> Indexer<Phase1State<'a>> for Phase1Indexer<'a> {
    const SIZE: usize = 64350 * CornerOrientationIndexer::SIZE; // 64350 is the number of unique symmetry classes over the EOS index
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, state: &Phase1State<'a>) -> usize {
        let eos = EOSIndexer.to_index(&(state.eo, state.esc));
        let (eos_class, sym) = self.eos_reduction_table.get(eos);
        let co_conj = self.co_symmetry_table.get(state.co, sym);
        eos_class * CornerOrientationIndexer::SIZE + co_conj
    }

    fn from_index(&self, _index: usize) -> Phase1State<'a> {
        unimplemented!("Phase1Indexer::from_index not needed for solving");
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
}

impl<'a> Indexer<Phase2State<'a>> for Phase2Indexer1<'a> {
    const SIZE: usize = 2520 * UDEdgePermutationIndexer::SIZE; // 2520 is the number of unique symmetry classes over the CP index
    const SOLVED_INDEX: usize = 0; // TODO: Calculate actual solved index

    fn to_index(&self, state: &Phase2State<'a>) -> usize {
        let (cp_class, sym) = self.cp_reduction_table.get(state.cp);
        let ud_conj = self.ud_symmetry_table.get(state.ud, sym);
        cp_class * UDEdgePermutationIndexer::SIZE + ud_conj
    }

    fn from_index(&self, _index: usize) -> Phase2State<'a> {
        unimplemented!("Phase2Indexer1::from_index is not implemented");
    }
}


#[derive(Clone, Copy)]
pub struct Phase2Indexer2;

impl<'a> Indexer<Phase2State<'a>> for Phase2Indexer2 {
    const SIZE: usize = CornerPermutationIndexer::SIZE * UDEdgePermutationIndexer::SIZE;
    const SOLVED_INDEX: usize = 0; // TODO: Calculate actual solved index

    fn to_index(&self, state: &Phase2State<'a>) -> usize {
        state.cp * ESlicePermutationIndexer::SIZE + state.esp
    }

    fn from_index(&self, _index: usize) -> Phase2State<'a> {
        unimplemented!("Phase2Indexer2::from_index is not implemented");
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
        for edge in EDGES {
            let orientation = orientation_cube.get_edge_orientation(edge);
            cube.set_edge_orientation(edge, orientation);
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

// struct MultiIndexer<I1: Indexer<Cube>, I2: Indexer<Cube>> {
//     pub indexer1: I1,
//     pub indexer2: I2,
// }

// impl<I1: Indexer<Cube>, I2: Indexer<Cube>> Indexer<Cube> for MultiIndexer<I1, I2> {
//     const SIZE: usize = I1::SIZE * I2::SIZE;
//     const SOLVED_INDEX: usize = I1::SOLVED_INDEX * I2::SIZE + I2::SOLVED_INDEX;

//     fn to_index(&self, cube: &Cube) -> usize {
//         let idx1 = self.indexer1.to_index(cube);
//         let idx2 = self.indexer2.to_index(cube);
//         idx1 * I2::SIZE + idx2
//     }

//     fn from_index(&self, _index: usize) -> Cube {
//         unimplemented!("MultiIndexer::from_index is not well defined");
//     }
// }

// impl<I1: Indexer<Cube>, I2: Indexer<Cube>> Indexer<(usize, usize)> for MultiIndexer<I1, I2> {
//     const SIZE: usize = I1::SIZE * I2::SIZE;
//     const SOLVED_INDEX: usize = I1::SOLVED_INDEX * I2::SIZE + I2::SOLVED_INDEX;

//     fn to_index(&self, state: &(usize, usize)) -> usize {
//         let idx1 = state.0;
//         let idx2 = state.1;
//         idx1 * I2::SIZE + idx2
//     }

//     fn from_index(&self, index: usize) -> (usize, usize) {
//         let idx1 = index / I2::SIZE;
//         let idx2 = index % I2::SIZE;
//         (idx1, idx2)
//     }
// }

// pub type EOSIndexer = MultiIndexer<EdgeOrientationIndexer, ESliceIndexer>;

// #[derive(Clone, Copy)]
// pub struct IndexedState<'a> {
//     idx: usize,
//     table: &'a IndexMoveTable,
// }

// impl<'a> IndexedState<'a> {
//     pub fn new(idx: usize, table: &'a IndexMoveTable) -> Self {
//         Self { idx, table }
//     }
// }

// // Impls for simple implementation and testing
// impl Moveable for IndexedState<'_> {
//     fn turn(&mut self, mv: Move) -> &mut Self {
//         self.idx = self.table.get(self.idx, mv);
//         self
//     }
// }

// impl<T: Indexer<Cube>> Indexer<IndexedState<'_>> for T {
//     const SIZE: usize = T::SIZE;
//     const SOLVED_INDEX: usize = T::SOLVED_INDEX;

//     fn to_index(&self, state: &IndexedState) -> usize {
//         state.idx
//     }

//     fn from_index(&self, _idx: usize) -> IndexedState<'static> {
//         unimplemented!("This is a helper struct for move tables and should not be used directly")
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use cube::Moveable;
    use std::sync::LazyLock;

    static TABLES: LazyLock<MoveTables> = LazyLock::new(|| {
        MoveTables::build()
    });

    #[test]
    fn phase1_state_solved() {
        let cube = Cube::new_solved();
        let state = Phase1State::from_cube(&cube, &TABLES);
        assert!(state.is_solved(), "Solved cube should be recognized as solved");
    }

    #[test]
    fn phase1_state_turn() {
        let cube = Cube::new_solved();
        let mut state = Phase1State::from_cube(&cube, &TABLES);
        state.turn(Move::R);
        assert!(!state.is_solved(), "After R move, should not be solved");
    }

    #[test]
    fn phase2_state_solved() {
        let cube = Cube::new_solved();
        let state = Phase2State::from_cube(&cube, &TABLES);
        assert!(state.is_solved(), "Solved cube should be recognized as solved");
    }
    
    #[test]
    fn phase2_state_turn() {
        let cube = Cube::new_solved();
        let mut state = Phase2State::from_cube(&cube, &TABLES);
        state.turn(Move::R2);
        assert!(!state.is_solved(), "After R2 move, should not be solved");
    }
}
