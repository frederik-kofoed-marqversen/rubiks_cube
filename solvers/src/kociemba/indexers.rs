use cube::math::*;
use cube::{Corner, Cube, Edge, CORNERS, EDGES};

pub trait Indexer<T> {
    const SIZE: usize;
    const SOLVED_INDEX: usize;

    /// Maps a cube state to an index in [0, SIZE).
    /// INVARIANT: Should map a solved state to `SOLVED_INDEX`.
    fn to_index(&self, cube: &T) -> usize;

    /// Maps an index in [0, SIZE) back to a cube state.
    /// Inverse of `to_index`.
    fn from_index(&self, index: usize) -> T;
}

const U_EDGES: [Edge; 4] = [Edge::UR, Edge::UB, Edge::UL, Edge::UF];
const E_SLICE: [Edge; 4] = [Edge::RF, Edge::RB, Edge::LB, Edge::LF];
const D_EDGES: [Edge; 4] = [Edge::DR, Edge::DB, Edge::DL, Edge::DF];

pub struct EdgeOrientationIndexer;
pub struct CornerOrientationIndexer;
pub struct CornerPermutationIndexer;
pub struct ESliceIndexer;
pub struct ESlicePermutationIndexer;
pub struct ESliceCombinationIndexer;
pub struct UEdgeIndexer;
pub struct DEdgeIndexer;
pub struct UDEdgePermutationIndexer;

impl Indexer<Cube> for EdgeOrientationIndexer {
    const SIZE: usize = 2048; // 2^11 possible orientations
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, cube: &Cube) -> usize {
        let mut index = 0;
        for &pos in EDGES[..11].iter() {
            index <<= 1; // *= 2;
            index |= cube.get_edge_orientation(pos) as usize;
        }
        index
    }

    fn from_index(&self, index: usize) -> Cube {
        let mut cube = Cube::new_solved();

        // Decode the 11 edge orientations from the index
        let mut remaining_index = index;
        let mut sum = 0;
        for &edge in EDGES[..11].iter().rev() {
            let bit = (remaining_index & 1) as u32;
            cube.set_edge_orientation(edge, bit);
            sum += bit;
            remaining_index >>= 1;
        }

        // 12th edge orientation determined by parity (must sum to even)
        cube.set_edge_orientation(EDGES[11], sum % 2);

        cube
    }
}

impl Indexer<Cube> for CornerOrientationIndexer {
    const SIZE: usize = 2187; // 3^7 possible orientations
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, cube: &Cube) -> usize {
        let mut index = 0;
        for &pos in CORNERS[..7].iter() {
            index *= 3;
            index += cube.get_corner_orientation(pos) as usize;
        }
        index
    }

    fn from_index(&self, index: usize) -> Cube {
        let mut cube = Cube::new_solved();

        // Decode the 7 corner orientations from the index (base-3 number)
        let mut remaining_index = index;
        let mut sum = 0;
        for &corner in CORNERS[..7].iter().rev() {
            let orientation = (remaining_index % 3) as u32;
            cube.set_corner_orientation(corner, orientation);
            sum += orientation;
            remaining_index /= 3;
        }

        // 8th corner orientation determined by parity (must sum to 0 mod 3)
        cube.set_corner_orientation(CORNERS[7], (3 - (sum % 3)) % 3);

        cube
    }
}

impl Indexer<Cube> for CornerPermutationIndexer {
    const SIZE: usize = 40320; // 8! possible permutations
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, cube: &Cube) -> usize {
        let mut corners = [Corner::URF; 8];
        for (i, &pos) in CORNERS.iter().enumerate() {
            corners[i] = cube.get_corner_type(pos);
        }
        let permutation: [usize; 8] = compute_permutation(&CORNERS, &corners);
        permutation_rank::<8>(&permutation)
    }

    fn from_index(&self, index: usize) -> Cube {
        let mut cube = Cube::new_solved();
        let permutation: [usize; 8] = permutation_unrank::<8>(index);
        for (i, &pos) in CORNERS.iter().enumerate() {
            cube.set_corner_type(pos, CORNERS[permutation[i]]);
        }
        cube
    }
}

const E_SLICE_INDEXER_EDGES: [Edge; 12] = [
    Edge::RF,
    Edge::RB,
    Edge::LB,
    Edge::LF,
    Edge::UR,
    Edge::UB,
    Edge::UL,
    Edge::UF,
    Edge::DR,
    Edge::DB,
    Edge::DL,
    Edge::DF,
];
/// Tracks the location of the 4 E-slice edges among all edge positions
/// as well as their mutual relative permutation.
impl Indexer<Cube> for ESliceIndexer {
    const SIZE: usize = 11880; // (12 choose 4) * 4! = 495 * 24
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, cube: &Cube) -> usize {
        let mut selected = [false; 12];
        let mut slice_edges = [E_SLICE[0]; 4];
        let mut k = 0; // Number of E-slice edges found so far
        for (i, &pos) in E_SLICE_INDEXER_EDGES.iter().enumerate() {
            let edge = cube.get_edge_type(pos);
            if E_SLICE.contains(&edge) {
                selected[i] = true;
                slice_edges[k] = edge;
                k += 1;
            }
        }
        let combination_index = combination_rank::<12, 4>(&selected);
        let permutation_index = permutation_rank::<4>(&compute_permutation(&E_SLICE, &slice_edges));

        let index = combination_index * 24 + permutation_index;
        index
    }

    fn from_index(&self, index: usize) -> Cube {
        let mut cube = Cube::new_solved();
        let combination_index = index / 24;
        let permutation_index = index % 24;

        let selected: [bool; 12] = combination_unrank::<12, 4>(combination_index);
        let permutation: [usize; 4] = permutation_unrank::<4>(permutation_index);

        // First permute the edges relative to each other
        let mut slice_edge_iter = (0..4).map(|i| E_SLICE[permutation[i]]);

        // Then place the slice edges in the positions indicated by the combination index
        let mut non_slice_edge_iter = E_SLICE_INDEXER_EDGES
            .iter()
            .filter(|&e| !E_SLICE.contains(e));
        for (i, &pos) in E_SLICE_INDEXER_EDGES.iter().enumerate() {
            if selected[i] {
                cube.set_edge_type(pos, slice_edge_iter.next().unwrap());
            } else {
                cube.set_edge_type(pos, *non_slice_edge_iter.next().unwrap());
            }
        }

        cube
    }
}

impl Indexer<Cube> for ESlicePermutationIndexer {
    const SIZE: usize = 24; // 4! possible permutations of the 4 E-slice edges
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, cube: &Cube) -> usize {
        ESliceIndexer.to_index(cube) % 24
    }

    fn from_index(&self, index: usize) -> Cube {
        ESliceIndexer.from_index(index)
    }
}

impl Indexer<Cube> for ESliceCombinationIndexer {
    const SIZE: usize = 495; // (12 choose 4) possible combinations of the 4 E-slice edges
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, cube: &Cube) -> usize {
        ESliceIndexer.to_index(cube) / 24
    }

    fn from_index(&self, index: usize) -> Cube {
        ESliceIndexer.from_index(index * 24)
    }
}

const U_EDGE_INDEXER_EDGES: [Edge; 12] = [
    Edge::UR,
    Edge::UB,
    Edge::UL,
    Edge::UF,
    Edge::RF,
    Edge::RB,
    Edge::LB,
    Edge::LF,
    Edge::DR,
    Edge::DB,
    Edge::DL,
    Edge::DF,
];
/// Tracks the location of the 4 U-face edges among all edge positions
/// as well as their mutual relative permutation.
impl Indexer<Cube> for UEdgeIndexer {
    const SIZE: usize = 11880; // (12 choose 4) * 4! = 495 * 24
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, cube: &Cube) -> usize {
        let mut selected = [false; 12];
        let mut u_edges = [U_EDGES[0]; 4];
        let mut k = 0; // Number of U edges found so far
        for (i, &pos) in U_EDGE_INDEXER_EDGES.iter().enumerate() {
            let edge = cube.get_edge_type(pos);
            if U_EDGES.contains(&edge) {
                selected[i] = true;
                u_edges[k] = edge;
                k += 1;
            }
        }
        let combination_index = combination_rank::<12, 4>(&selected);
        let permutation_index = permutation_rank::<4>(&compute_permutation(&U_EDGES, &u_edges));
        combination_index * 24 + permutation_index
    }

    fn from_index(&self, index: usize) -> Cube {
        let mut cube = Cube::new_solved();
        let combination_index = index / 24;
        let permutation_index = index % 24;

        let selected: [bool; 12] = combination_unrank::<12, 4>(combination_index);
        let permutation: [usize; 4] = permutation_unrank::<4>(permutation_index);

        // First permute the U edges relative to each other
        let mut u_edge_iter = (0..4).map(|i| U_EDGES[permutation[i]]);

        // Then place the U edges in the positions indicated by the combination index
        let mut non_u_edge_iter = U_EDGE_INDEXER_EDGES
            .iter()
            .filter(|&e| !U_EDGES.contains(e));
        for (i, &pos) in U_EDGE_INDEXER_EDGES.iter().enumerate() {
            if selected[i] {
                cube.set_edge_type(pos, u_edge_iter.next().unwrap());
            } else {
                cube.set_edge_type(pos, *non_u_edge_iter.next().unwrap());
            }
        }

        cube
    }
}

const D_EDGE_INDEXER_EDGES: [Edge; 12] = [
    Edge::DR,
    Edge::DB,
    Edge::DL,
    Edge::DF,
    Edge::UR,
    Edge::UB,
    Edge::UL,
    Edge::UF,
    Edge::RF,
    Edge::RB,
    Edge::LB,
    Edge::LF,
];
impl Indexer<Cube> for DEdgeIndexer {
    const SIZE: usize = 11880; // (12 choose 4) * 4! = 495 * 24
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, cube: &Cube) -> usize {
        let mut selected = [false; 12];
        let mut d_edges = [D_EDGES[0]; 4];
        let mut k = 0; // Number of D edges found so far
        for (i, &pos) in D_EDGE_INDEXER_EDGES.iter().enumerate() {
            let edge = cube.get_edge_type(pos);
            if D_EDGES.contains(&edge) {
                selected[i] = true;
                d_edges[k] = edge;
                k += 1;
            }
        }
        let combination_index = combination_rank::<12, 4>(&selected);
        let permutation_index = permutation_rank::<4>(&compute_permutation(&D_EDGES, &d_edges));

        let index = combination_index * 24 + permutation_index;
        index
    }

    fn from_index(&self, index: usize) -> Cube {
        let mut cube = Cube::new_solved();
        let combination_index = index / 24;
        let permutation_index = index % 24;

        let selected: [bool; 12] = combination_unrank::<12, 4>(combination_index);
        let permutation: [usize; 4] = permutation_unrank::<4>(permutation_index);

        // First permute the D edges relative to each other
        let mut d_edge_iter = (0..4).map(|i| D_EDGES[permutation[i]]);

        // Then place the D edges in the positions indicated by the combination index
        let mut non_d_edge_iter = D_EDGE_INDEXER_EDGES
            .iter()
            .filter(|&e| !D_EDGES.contains(e));
        for (i, &pos) in D_EDGE_INDEXER_EDGES.iter().enumerate() {
            if selected[i] {
                cube.set_edge_type(pos, d_edge_iter.next().unwrap());
            } else {
                cube.set_edge_type(pos, *non_d_edge_iter.next().unwrap());
            }
        }

        cube
    }
}

const UD_EDGES: [Edge; 8] = [
    Edge::UR,
    Edge::UB,
    Edge::UL,
    Edge::UF,
    Edge::DR,
    Edge::DB,
    Edge::DL,
    Edge::DF,
];

const UD_EDGE_INDEXER_EDGES: [Edge; 12] = [
    Edge::UR,
    Edge::UB,
    Edge::UL,
    Edge::UF,
    Edge::DR,
    Edge::DB,
    Edge::DL,
    Edge::DF,
    Edge::RF,
    Edge::RB,
    Edge::LB,
    Edge::LF,
];

impl Indexer<Cube> for UDEdgePermutationIndexer {
    const SIZE: usize = 40320; // 8! possible permutations of the 8 U and D edges
    const SOLVED_INDEX: usize = 0;

    fn to_index(&self, cube: &Cube) -> usize {
        let mut d_edges = [UD_EDGES[0]; 8];
        let mut k = 0;
        for (_i, &pos) in UD_EDGE_INDEXER_EDGES.iter().enumerate() {
            let edge = cube.get_edge_type(pos);
            if UD_EDGES.contains(&edge) {
                d_edges[k] = edge;
                k += 1;
            }
        }
        let permutation_index = permutation_rank::<8>(&compute_permutation(&UD_EDGES, &d_edges));

        permutation_index
    }

    fn from_index(&self, index: usize) -> Cube {
        let mut cube = Cube::new_solved();
        let permutation: [usize; 8] = permutation_unrank::<8>(index);

        for (i, &pos) in UD_EDGES.iter().enumerate() {
            cube.set_edge_type(pos, UD_EDGES[permutation[i]]);
        }
        cube
    }
}

#[macro_export]
macro_rules! test_indexer {
    ($mod_name:ident, $indexer_type:ty, $indexer:expr, $state_type:ty, $solved_state:expr) => {
        mod $mod_name {
            use super::*;

            #[test]
            fn solved_index() {
                let indexer = $indexer;
                assert_eq!(
                    indexer.to_index(&$solved_state),
                    <$indexer_type as Indexer<$state_type>>::SOLVED_INDEX
                );
            }

            #[test]
            fn consistency() {
                const SIZE: usize = <$indexer_type as Indexer<$state_type>>::SIZE;
                let indexer = $indexer;
                let mut seen = vec![false; SIZE];
                
                for i in 0..SIZE {
                    let state: $state_type = indexer.from_index(i);
                    let j = indexer.to_index(&state);
                    
                    assert!(j < SIZE, "Index {} out of bounds for SIZE {}", j, SIZE);
                    assert_eq!(i, j, "Failed for index {}", i);
                    assert!(!seen[j], "Index {} produced twice", j);
                    seen[j] = true;
                }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    test_indexer!(eo, EdgeOrientationIndexer, EdgeOrientationIndexer, Cube, Cube::new_solved());
    test_indexer!(co, CornerOrientationIndexer, CornerOrientationIndexer, Cube, Cube::new_solved());
    test_indexer!(cp, CornerPermutationIndexer, CornerPermutationIndexer, Cube, Cube::new_solved());
    test_indexer!(es, ESliceIndexer, ESliceIndexer, Cube, Cube::new_solved());
    test_indexer!(esc, ESliceCombinationIndexer, ESliceCombinationIndexer, Cube, Cube::new_solved());
    test_indexer!(esp, ESlicePermutationIndexer, ESlicePermutationIndexer, Cube, Cube::new_solved());
    test_indexer!(ue, UEdgeIndexer, UEdgeIndexer, Cube, Cube::new_solved());
    test_indexer!(de, DEdgeIndexer, DEdgeIndexer, Cube, Cube::new_solved());
    test_indexer!(ude, UDEdgePermutationIndexer, UDEdgePermutationIndexer, Cube, Cube::new_solved());
}
