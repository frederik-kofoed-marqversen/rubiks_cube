use std::f32::consts::E;

use super::cube::{Corner, Cube, Edge, CORNERS, EDGES};
use crate::math::*;

pub trait Indexer {
    const SIZE: usize;

    fn to_index(cube: &Cube) -> usize;
    fn from_index(index: usize) -> Cube;
}

const U_EDGES: [Edge; 4] = [Edge::UR, Edge::UB, Edge::UL, Edge::UF];
const D_EDGES: [Edge; 4] = [Edge::DR, Edge::DB, Edge::DL, Edge::DF];
const E_SLICE: [Edge; 4] = [Edge::RF, Edge::RB, Edge::LB, Edge::LF];
const NON_E_SLICE: [Edge; 8] = [
    Edge::UR,
    Edge::UB,
    Edge::UL,
    Edge::UF,
    Edge::DR,
    Edge::DB,
    Edge::DL,
    Edge::DF,
];

pub struct EdgeOrientationIndexer;
pub struct CornerOrientationIndexer;
pub struct ESliceIndexer;
pub struct CornerPermutationIndexer;
pub struct UEdgePermutationIndexer;
pub struct DEdgePermutationIndexer;
pub struct ESlicePermutationIndexer;

impl Indexer for EdgeOrientationIndexer {
    const SIZE: usize = 2048; // 2^11 possible orientations

    fn to_index(cube: &Cube) -> usize {
        let mut index = 0;
        for &pos in EDGES[..11].iter() {
            index <<= 1; // *= 2;
            index |= cube.get_edge_orientation(pos) as usize;
        }
        index
    }

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::solved();

        // Decode the 11 edge orientations from the index
        let mut remaining_index = index;
        let mut sum = 0;
        for &edge in EDGES[..11].iter().rev() {
            let bit = (remaining_index & 1) as u8;
            cube.set_edge_orientation(edge, bit);
            sum += bit;
            remaining_index >>= 1;
        }

        // 12th edge orientation determined by parity (must sum to even)
        cube.set_edge_orientation(EDGES[11], sum % 2);

        cube
    }
}

impl Indexer for CornerOrientationIndexer {
    const SIZE: usize = 2187; // 3^7 possible orientations

    fn to_index(cube: &Cube) -> usize {
        let mut index = 0;
        for &pos in CORNERS[..7].iter() {
            index *= 3;
            index += cube.get_corner_orientation(pos) as usize;
        }
        index
    }

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::solved();

        // Decode the 7 corner orientations from the index (base-3 number)
        let mut remaining_index = index;
        let mut sum = 0;
        for &corner in CORNERS[..7].iter().rev() {
            let orientation = (remaining_index % 3) as u8;
            cube.set_corner_orientation(corner, orientation);
            sum += orientation;
            remaining_index /= 3;
        }

        // 8th corner orientation determined by parity (must sum to 0 mod 3)
        cube.set_corner_orientation(CORNERS[7], (3 - (sum % 3)) % 3);

        cube
    }
}

/// Tracks which 4 of the 12 edge positions currently contain E-slice edges.
impl Indexer for ESliceIndexer {
    const SIZE: usize = 495; // (12 choose 4) possible combinations

    fn to_index(cube: &Cube) -> usize {
        // Find positions of the 4 E-slice edges
        let mut selected = [false; 12];
        for (i, &pos) in EDGES.iter().enumerate() {
            let edge = cube.get_edge_type(pos);
            selected[i] = E_SLICE.contains(&edge);
        }
        // Index given by combination rank
        return combination_rank::<12, 4>(&selected);
    }

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::solved();
        let selected: [bool; 12] = combination_unrank::<12, 4>(index);

        let mut e_slice_iter = E_SLICE.iter();
        let mut non_e_slice_iter = NON_E_SLICE.iter();

        for (i, &pos) in EDGES.iter().enumerate() {
            if selected[i] {
                cube.set_edge_type(pos, *e_slice_iter.next().unwrap());
            } else {
                cube.set_edge_type(pos, *non_e_slice_iter.next().unwrap());
            }
        }

        cube
    }
}

impl Indexer for CornerPermutationIndexer {
    const SIZE: usize = 40320; // 8! possible permutations

    fn to_index(cube: &Cube) -> usize {
        let mut corners = [Corner::URF; 8];
        for (i, &pos) in CORNERS.iter().enumerate() {
            corners[i] = cube.get_corner_type(pos);
        }
        let permutation: [usize; 8] = compute_permutation(&CORNERS, &corners);
        permutation_rank::<8>(&permutation)
    }

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::solved();
        let permutation: [usize; 8] = permutation_unrank::<8>(index);
        for (i, &pos) in CORNERS.iter().enumerate() {
            cube.set_corner_type(pos, CORNERS[permutation[i]]);
        }
        cube
    }
}

impl Indexer for ESlicePermutationIndexer {
    const SIZE: usize = 24; // 4! permutations of the 4 E-slice edges

    fn to_index(cube: &Cube) -> usize {
        let mut edges = [Edge::UR; 4];
        for (i, &edge) in E_SLICE.iter().enumerate() {
            edges[i] = cube.get_edge_type(edge);
        }
        let permutation: [usize; 4] = compute_permutation(&E_SLICE, &edges);
        permutation_rank::<4>(&permutation)
    }

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::solved();
        let permutation: [usize; 4] = permutation_unrank::<4>(index);
        for (i, &edge) in E_SLICE.iter().enumerate() {
            cube.set_edge_type(edge, E_SLICE[permutation[i]]);
        }
        cube
    }
}

/// Tracks the location of the 4 U-face edges among all edge positions
/// as well as their mutual relative permutation.
impl Indexer for UEdgePermutationIndexer {
    const SIZE: usize = 11880; // (12 choose 4) * 4! = 495 * 24

    fn to_index(cube: &Cube) -> usize {
        let mut selected = [false; 12];
        let mut u_edges = [Edge::UR; 4];
        let mut k = 0; // Number of U edges found so far
        for (i, &pos) in EDGES.iter().enumerate() {
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

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::solved();
        let combination_index = index / 24;
        let permutation_index = index % 24;

        let selected: [bool; 12] = combination_unrank::<12, 4>(combination_index);
        let permutation: [usize; 4] = permutation_unrank::<4>(permutation_index);

        // First permute the U edges relative to each other
        let mut u_edges_perm = [Edge::UR; 4];
        for (i, &pos) in U_EDGES.iter().enumerate() {
            u_edges_perm[i] = U_EDGES[permutation[i]];
        }

        // Then place the U edges in the positions indicated by the combination index
        let mut u_edge_iter = u_edges_perm.iter();
        let mut non_u_edge_iter = EDGES.iter().filter(|&&e| !U_EDGES.contains(&e));
        for (i, &pos) in EDGES.iter().enumerate() {
            if selected[i] {
                cube.set_edge_type(pos, *u_edge_iter.next().unwrap());
            } else {
                cube.set_edge_type(pos, *non_u_edge_iter.next().unwrap());
            }
        }

        cube
    }
}

impl Indexer for DEdgePermutationIndexer {
    const SIZE: usize = 11880; // (12 choose 4) * 4! = 495 * 24

    fn to_index(cube: &Cube) -> usize {
        let mut selected = [false; 12];
        let mut d_edges = [Edge::DR; 4];
        let mut k = 0; // Number of D edges found so far
        for (i, &pos) in EDGES.iter().enumerate() {
            let edge = cube.get_edge_type(pos);
            if D_EDGES.contains(&edge) {
                selected[i] = true;
                d_edges[k] = edge;
                k += 1;
            }
        }
        let combination_index = combination_rank::<12, 4>(&selected);
        let permutation_index = permutation_rank::<4>(&compute_permutation(&D_EDGES, &d_edges));
        combination_index * 24 + permutation_index
    }

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::solved();
        let combination_index = index / 24;
        let permutation_index = index % 24;

        let selected: [bool; 12] = combination_unrank::<12, 4>(combination_index);
        let permutation: [usize; 4] = permutation_unrank::<4>(permutation_index);

        // First permute the D edges relative to each other
        let mut d_edges_perm = [Edge::DR; 4];
        for (i, &pos) in D_EDGES.iter().enumerate() {
            d_edges_perm[i] = D_EDGES[permutation[i]];
        }

        // Then place the D edges in the positions indicated by the combination index
        let mut d_edge_iter = d_edges_perm.iter();
        let mut non_d_edge_iter = EDGES.iter().filter(|&&e| !D_EDGES.contains(&e));
        for (i, &pos) in EDGES.iter().enumerate() {
            if selected[i] {
                cube.set_edge_type(pos, *d_edge_iter.next().unwrap());
            } else {
                cube.set_edge_type(pos, *non_d_edge_iter.next().unwrap());
            }
        }

        cube
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type EO = EdgeOrientationIndexer;
    type CO = CornerOrientationIndexer;
    type ES = ESliceIndexer;
    type CP = CornerPermutationIndexer;
    type ESP = ESlicePermutationIndexer;
    type UEP = UEdgePermutationIndexer;
    type DEP = DEdgePermutationIndexer;

    #[test]
    fn eo_zero_index() {
        assert_eq!(EO::to_index(&Cube::solved()), 0);
    }

    #[test]
    fn co_zero_index() {
        assert_eq!(CO::to_index(&Cube::solved()), 0);
    }

    #[test]
    fn es_zero_index() {
        assert_eq!(ES::to_index(&Cube::solved()), 0);
    }

    #[test]
    fn cp_zero_index() {
        assert_eq!(CP::to_index(&Cube::solved()), 0);
    }

    #[test]
    fn esp_zero_index() {
        assert_eq!(ESP::to_index(&Cube::solved()), 0);
    }

    #[test]
    fn uep_zero_index() {
        assert_eq!(UEP::to_index(&Cube::solved()), 0);
    }

    #[test]
    fn dep_zero_index() {
        assert_eq!(DEP::to_index(&Cube::solved()), 0);
    }

    #[test]
    fn eo_indexing_consistency() {
        for i in 0..EO::SIZE {
            let test = EO::to_index(&EO::from_index(i));
            assert_eq!(i, test, "Failed for index {i}");
        }
    }

    #[test]
    fn co_indexing_consistency() {
        for i in 0..CO::SIZE {
            let test = CO::to_index(&CO::from_index(i));
            assert_eq!(i, test, "Failed for index {i}");
        }
    }

    #[test]
    fn es_indexing_consistency() {
        for i in 0..ES::SIZE {
            let test = ES::to_index(&ES::from_index(i));
            assert_eq!(i, test, "Failed for index {i}");
        }
    }

    #[test]
    fn cp_indexing_consistency() {
        for i in 0..CP::SIZE {
            let test = CP::to_index(&CP::from_index(i));
            assert_eq!(i, test, "Failed for index {i}");
        }
    }

    #[test]
    fn esp_indexing_consistency() {
        for i in 0..ESP::SIZE {
            let test = ESP::to_index(&ESP::from_index(i));
            assert_eq!(i, test, "Failed for index {i}");
        }
    }

    #[test]
    fn uep_indexing_consistency() {
        for i in 0..UEP::SIZE {
            let test = UEP::to_index(&UEP::from_index(i));
            assert_eq!(i, test, "Failed for index {i}");
        }
    }

    #[test]
    fn dep_indexing_consistency() {
        for i in 0..DEP::SIZE {
            let test = DEP::to_index(&DEP::from_index(i));
            assert_eq!(i, test, "Failed for index {i}");
        }
    }
}
