use super::cube::{Corner, Cube, Edge, CORNERS, EDGES};
use crate::math::precompute_binomials;

const BINOM: [[usize; 5]; 13] = precompute_binomials();

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

// Combination rank of the 4 edges currently in the E slice
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
                index += BINOM[i][k];
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
    fn eo_indexing() {
        for i in 0..EO::SIZE {
            let test = EO::to_index(&EO::from_index(i));
            assert_eq!(i, test, "Edge Orientation Indexing failed for index {i}");
        }
    }

    #[test]
    fn co_indexing() {
        for i in 0..CO::SIZE {
            let test = CO::to_index(&CO::from_index(i));
            assert_eq!(i, test, "Corner Orientation Indexing failed for index {i}");
        }
    }

    #[test]
    fn es_indexing() {
        for i in 0..ES::SIZE {
            let test = ES::to_index(&ES::from_index(i));
            assert_eq!(i, test, "E-Slice Indexing failed for index {i}");
        }
    }
}
