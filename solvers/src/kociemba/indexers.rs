use cube::{Corner, Cube, Edge, CORNERS, EDGES};
use cube::math::*;

pub trait Indexer<T> {
    const SIZE: usize;
    const SOLVED_INDEX: usize = 0;

    /// Maps a cube state to an index in [0, SIZE).
    /// INVARIANT: Should map a solved state to `SOLVED_INDEX`.
    fn to_index(cube: &T) -> usize;

    /// Maps an index in [0, SIZE) back to a cube state.
    /// Inverse of `to_index`.
    fn from_index(index: usize) -> T;
}

const U_EDGES: [Edge; 4] = [Edge::UR, Edge::UB, Edge::UL, Edge::UF];
const E_SLICE: [Edge; 4] = [Edge::RF, Edge::RB, Edge::LB, Edge::LF];
const D_EDGES: [Edge; 4] = [Edge::DR, Edge::DB, Edge::DL, Edge::DF];

pub struct EdgeOrientationIndexer;
pub struct CornerOrientationIndexer;
pub struct CornerPermutationIndexer;
pub struct ESliceIndexer;
pub struct UEdgeIndexer;
pub struct DEdgeIndexer;

impl Indexer<Cube> for EdgeOrientationIndexer {
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

    fn to_index(cube: &Cube) -> usize {
        let mut index = 0;
        for &pos in CORNERS[..7].iter() {
            index *= 3;
            index += cube.get_corner_orientation(pos) as usize;
        }
        index
    }

    fn from_index(index: usize) -> Cube {
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

    fn to_index(cube: &Cube) -> usize {
        let mut corners = [Corner::URF; 8];
        for (i, &pos) in CORNERS.iter().enumerate() {
            corners[i] = cube.get_corner_type(pos);
        }
        let permutation: [usize; 8] = compute_permutation(&CORNERS, &corners);
        permutation_rank::<8>(&permutation)
    }

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::new_solved();
        let permutation: [usize; 8] = permutation_unrank::<8>(index);
        for (i, &pos) in CORNERS.iter().enumerate() {
            cube.set_corner_type(pos, CORNERS[permutation[i]]);
        }
        cube
    }
}

const E_SLICE_INDEXER_EDGES: [Edge; 12] = [
    Edge::RF, Edge::RB, Edge::LB, Edge::LF,
    Edge::UR, Edge::UB, Edge::UL, Edge::UF,
    Edge::DR, Edge::DB, Edge::DL, Edge::DF,
];
/// Tracks the location of the 4 E-slice edges among all edge positions
/// as well as their mutual relative permutation.
impl Indexer<Cube> for ESliceIndexer {
    const SIZE: usize = 11880; // (12 choose 4) * 4! = 495 * 24

    fn to_index(cube: &Cube) -> usize {
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

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::new_solved();
        let combination_index = index / 24;
        let permutation_index = index % 24;

        let selected: [bool; 12] = combination_unrank::<12, 4>(combination_index);
        let permutation: [usize; 4] = permutation_unrank::<4>(permutation_index);

        // First permute the edges relative to each other
        let mut slice_edge_iter = (0..4).map(|i| E_SLICE[permutation[i]]);

        // Then place the slice edges in the positions indicated by the combination index
        let mut non_slice_edge_iter = E_SLICE_INDEXER_EDGES.iter().filter(|&e| !E_SLICE.contains(e));
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

const U_EDGE_INDEXER_EDGES: [Edge; 12] = [
    Edge::UR, Edge::UB, Edge::UL, Edge::UF,
    Edge::RF, Edge::RB, Edge::LB, Edge::LF,
    Edge::DR, Edge::DB, Edge::DL, Edge::DF,
];
/// Tracks the location of the 4 U-face edges among all edge positions
/// as well as their mutual relative permutation.
impl Indexer<Cube> for UEdgeIndexer {
    const SIZE: usize = 11880; // (12 choose 4) * 4! = 495 * 24

    fn to_index(cube: &Cube) -> usize {
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

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::new_solved();
        let combination_index = index / 24;
        let permutation_index = index % 24;

        let selected: [bool; 12] = combination_unrank::<12, 4>(combination_index);
        let permutation: [usize; 4] = permutation_unrank::<4>(permutation_index);

        // First permute the U edges relative to each other
        let mut u_edge_iter = (0..4).map(|i| U_EDGES[permutation[i]]);

        // Then place the U edges in the positions indicated by the combination index
        let mut non_u_edge_iter = U_EDGE_INDEXER_EDGES.iter().filter(|&e| !U_EDGES.contains(e));
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
    Edge::DR, Edge::DB, Edge::DL, Edge::DF,
    Edge::UR, Edge::UB, Edge::UL, Edge::UF,
    Edge::RF, Edge::RB, Edge::LB, Edge::LF,
];
impl Indexer<Cube> for DEdgeIndexer {
    const SIZE: usize = 11880; // (12 choose 4) * 4! = 495 * 24

    fn to_index(cube: &Cube) -> usize {
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

    fn from_index(index: usize) -> Cube {
        let mut cube = Cube::new_solved();
        let combination_index = index / 24;
        let permutation_index = index % 24;

        let selected: [bool; 12] = combination_unrank::<12, 4>(combination_index);
        let permutation: [usize; 4] = permutation_unrank::<4>(permutation_index);

        // First permute the D edges relative to each other
        let mut d_edge_iter = (0..4).map(|i| D_EDGES[permutation[i]]);

        // Then place the D edges in the positions indicated by the combination index
        let mut non_d_edge_iter = D_EDGE_INDEXER_EDGES.iter().filter(|&e| !D_EDGES.contains(e));
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

// impl Indexer for UDEdgePermutationIndexer {
//     const SIZE: usize = 40320; // 8! possible permutations of the 8 U and D edges

//     fn to_index(cube: &Cube) -> usize {
//         unimplemented!()
//     }

//     fn from_index(index: usize) -> Cube {
//         unimplemented!()
//     }

// def get_ud_edges(self):
//     """Get the permutation of the 8 U and D edges.
//         ud_edges undefined in phase 1, 0 <= ud_edges < 40320 in phase 2, ud_edges = 0 for solved cube."""
//     perm = self.ep[0:8]  # duplicate first 8 elements of ep
//     b = 0
//     for j in range(Ed.DB, Ed.UR, -1):
//         k = 0
//         while perm[j] != j:
//             rotate_left(perm, 0, j)
//             k += 1
//         b = (j + 1) * b + k
//     return b

// def set_ud_edges(self, idx):
//     # positions of FR FL BL BR edges are not affected
//     for i in list(Ed)[0:8]:
//         self.ep[i] = i
//     for j in list(Ed)[0:8]:
//         k = idx % (j + 1)
//         idx //= j + 1
//         while k > 0:
//             rotate_right(self.ep, 0, j)
//             k -= 1
// }

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_indexer {
        ($mod_name:ident, $indexer:ty) => {
            mod $mod_name {
                use super::*;

                #[test]
                fn solved_index() {
                    assert_eq!(
                        <$indexer as Indexer<Cube>>::to_index(&Cube::new_solved()),
                        <$indexer as Indexer<Cube>>::SOLVED_INDEX
                    );
                }

                #[test]
                fn consistency() {
                    for i in 0..<$indexer as Indexer<Cube>>::SIZE {
                        let test = <$indexer as Indexer<Cube>>::to_index(&<$indexer as Indexer<Cube>>::from_index(i));
                        assert_eq!(i, test, "Failed for index {}", i);
                    }
                }
            }
        };
    }

    test_indexer!(eo, EdgeOrientationIndexer);
    test_indexer!(co, CornerOrientationIndexer);
    test_indexer!(cp, CornerPermutationIndexer);
    test_indexer!(es, ESliceIndexer);
    test_indexer!(ue, UEdgeIndexer);
    test_indexer!(de, DEdgeIndexer);
}
