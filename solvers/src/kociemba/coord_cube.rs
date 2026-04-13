use super::cube::Cube;
use super::indexers::*;

#[derive(Clone, Copy, PartialEq)]
pub struct CoordinateCube {
    pub eo: usize,
    pub co: usize,
    pub cp: usize,
    pub es: usize,
    pub ue: usize,
    pub de: usize,
}

impl CoordinateCube {
    pub fn from_cube(cube: &Cube) -> Self {
        Self {
            eo: EdgeOrientationIndexer::to_index(cube),
            co: CornerOrientationIndexer::to_index(cube),
            cp: CornerPermutationIndexer::to_index(cube),
            es: ESliceIndexer::to_index(cube),
            ue: UEdgeIndexer::to_index(cube),
            de: DEdgeIndexer::to_index(cube),
        }
    }

    pub fn solved() -> Self {
        Self {
            eo: 0,
            co: 0,
            cp: 0,
            es: 0,
            ue: 0,
            de: 0,
        }
    }

    pub fn is_solved(self) -> bool {
        self == Self::solved()
    }
}
