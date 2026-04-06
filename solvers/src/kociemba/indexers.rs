use super::cube::{Cube, Edge, CORNERS};
use crate::math::precompute_binomials;

const BINOM: [[usize; 5]; 13] = precompute_binomials();
const E_SLICE: [Edge; 4] = [Edge::RF, Edge::RB, Edge::LB, Edge::LF];
const NON_E_SLICE: [Edge; 8] = [
    Edge::UR, Edge::UB, Edge::UL, Edge::UF,
    Edge::DR, Edge::DB, Edge::DL, Edge::DF,
];
// We define a local order of edges for indexing the E-slice combination with
// the 4 E-slice edges appearing first. This guarantees that the solved cube
// gets ES index = 0.
const EDGES: [Edge; 12] = [
    Edge::RF, Edge::RB, Edge::LB, Edge::LF,
    Edge::UR, Edge::UB, Edge::UL, Edge::UF,
    Edge::DR, Edge::DB, Edge::DL, Edge::DF,
];

pub trait Indexer {
    const SIZE: usize;

    fn to_index(cube: &Cube) -> usize;
    fn from_index(index: usize) -> Cube;
}

pub struct EdgeOrientationIndexer;
pub struct CornerOrientationIndexer;
pub struct ESliceIndexer;

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

/// E-Slice Position Indexer
/// 
/// Tracks which 4 of the 12 edge positions currently contain E-slice edges.
/// There are C(12,4) = 495 possible combinations.
///
/// EDGES array defines the coordinate system: E-slice positions come first (0-3),
/// ensuring the solved cube (E-slice edges at positions 0-3) maps to index 0.
///
/// Uses combination ranking: given positions [p₀, p₁, p₂, p₃] in sorted order,
/// index = C(p₀,1) + C(p₁,2) + C(p₂,3) + C(p₃,4)
///
/// Examples:
///   Solved cube: positions [0,1,2,3] → C(0,1)+C(1,2)+C(2,3)+C(3,4) = 0+0+0+0 = 0
///   Positions [0,1,2,4]: C(0,1)+C(1,2)+C(2,3)+C(4,4) = 0+0+0+1 = 1
impl Indexer for ESliceIndexer {
    const SIZE: usize = 495; // (12 choose 4) possible combinations

    fn to_index(cube: &Cube) -> usize {
        let mut index = 0;
        let mut k = 0; // Number of E-slice edges found so far

        for (i, &pos) in EDGES.iter().enumerate() {
            if k == 4 {
                break;
            }

            let edge = cube.get_edge_type(pos);
            if E_SLICE.contains(&edge) {
                k += 1;
                index += BINOM[i][k]; // C(pos_i, i+1)
            }
        }

        index
    }

    fn from_index(mut index: usize) -> Cube {
        let mut cube = Cube::solved();

        let mut e_slice_iter = E_SLICE.iter();
        let mut non_e_slice_iter = NON_E_SLICE.iter();
        
        let mut r = 4; // Remaining E-slice edges to place
        for (i, &pos) in EDGES.iter().enumerate().rev() {
            let b = BINOM[i][r];
            if r > 0 && index >= b {
                // This position contains an E-slice edge
                cube.set_edge_type(pos, *e_slice_iter.next().unwrap());
                index -= b;
                r -= 1;
            } else {
                // Position contains a non E-slice edge
                cube.set_edge_type(pos, *non_e_slice_iter.next().unwrap());
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
}
