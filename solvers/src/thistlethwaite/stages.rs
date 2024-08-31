use super::cube::{Cube, Move, Edge, Corner, EDGES, CORNERS};

pub trait Stage<'a> {
    const FILEPATH: &'a str;
    const MOVE_POOL: &'a [Move];
    const SIZE: usize;
    fn indexer(cube: &Cube) -> usize;
}

pub struct G1;
impl<'a> Stage<'a> for G1 {
    const FILEPATH: &'a str = &"./data/g1.dat";
    const SIZE: usize = 2048;
    const MOVE_POOL: &'a [Move] = &[
        Move::U, Move::Up, Move::U2,
        Move::L, Move::Lp, Move::L2,
        Move::D, Move::Dp, Move::D2,
        Move::R, Move::Rp, Move::R2,
        Move::F, Move::Fp, Move::F2,
        Move::B, Move::Bp, Move::B2,
    ];

    /**The index into the table of an edge orientation configuration given by `cube`.
     * We treat the array of edge orientations as a base-2 number. Only the first 11
     * edges are needed to define a unique index (even parity constraint). 
     */
    fn indexer(cube: &Cube) -> usize {
        // Edge orientation index
        EDGES[..11]
            .iter()
            .enumerate()
            .map(|(i, edge)| 2_usize.pow(i as u32) * cube.get_edge_orientation(edge) as usize)
            .sum()
    }
}

pub struct G2;
impl<'a> Stage<'a> for G2 {
    const FILEPATH: &'a str = &"./data/g2.dat";
    const SIZE: usize = 1082565;
    const MOVE_POOL: &'a [Move] = &[
        Move::U, Move::Up, Move::U2,
        Move::L, Move::Lp, Move::L2,
        Move::D, Move::Dp, Move::D2,
        Move::R, Move::Rp, Move::R2,
        Move::F2,
        Move::B2,
    ];

    /** The states reachable using G3 (⊆ G2), have all corners oriented and the 4 E-slice 
     * edges positioned somewhere in the E-slice.
     * We must compute a unique index for each corner orientation. Only 7 of the 8 corners 
     * has to be considered. That is 3^7=2187 possibilities. We also must compute an index
     * for each possible configuration of the position of the 4 E-slice edges. That gives 
     * (12 choose 4) = 495. All in all 3^7 * (12 choose 4) = 1082565 indices. 
     */
    fn indexer(cube: &Cube) -> usize {
        // We treat the array of corner orientations as a base-3 number.
        let corner_orientations = CORNERS[..7].iter()
            .map(|corner| cube.get_corner_orientation(corner));
        let corner_orientation_index = corner_orientations.enumerate()
            .map(|(i, n)| 3_usize.pow(i as u32) * n as usize)
            .sum::<usize>();

        const E_SLICE_EDGES: [Edge; 4] = [Edge::RF, Edge::RB, Edge::LB, Edge::LF];
        let e_slice_edge_positions = E_SLICE_EDGES.map(|edge| *cube.get_edge_position(&edge));
        let e_slice_edges_index = combination_rank(&e_slice_edge_positions, &EDGES);

        let index = corner_orientation_index * 495 + e_slice_edges_index;
        return index
    }
}

pub struct G3;
impl<'a> Stage<'a> for G3 {
    const FILEPATH: &'a str = &"./data/g3.dat";
    const SIZE: usize = 352800;
    const MOVE_POOL: &'a [Move] = &[
        Move::U, Move::Up, Move::U2,
        Move::L2,
        Move::D, Move::Dp, Move::D2,
        Move::R2,
        Move::F2,
        Move::B2,
    ];

    /**The states reachable using G4 (⊆ G3), have two tetrads (sets of four) of corners that
     * are disjoint and cannot be mixed by elements of G4. Also the tetrad twist of both
     * tetrads must be fixed. 
     * 
     * We use a stronger condition where both tetrads are split into two pairs. The goal is to 
     * correctly place all pairs, although we do not care if the two corners in a pair are 
     * swapped. That is (8 choose 2) * (6 choose 2) * (4 choose 2) = 2520 configurations. This 
     * strategy is taken from: 
     * https://www.stefan-pochmann.info/spocc/other_stuff/tools/solver_thistlethwaite/solver_thistlethwaite.txt
     * 
     * G4 reachable states also have even swap parity of corners (as well as edges, but these
     * parities are always equal). Thus, the swap parity is also tracked. This is a further 2 
     * configurations.
     * 
     * Finally, edges in each of the three slices (E, S, and M) are disjoint sets. Arbitrarily 
     * we track the 4 M-slice edges on the remaining 8 positions (the E-slice edges are placed 
     * in G3, and the S-slice edges must necessarily be placed when both E-, and M-slice edges 
     * are). This is (8 choose 4) = 70 configurations.
     * 
     * All in all this gives 2520 * 70 * 2 = 352800 configurations
    */
    fn indexer(cube: &Cube) -> usize {
        // First 4 and last 4 each make up a tetrad. First 2 and last 2 in each tetrad is a pair.
        const PAIRED_CORNERS: [Corner; 8] = [
            Corner::URF, Corner::ULB,
            Corner::DRB, Corner::DLF,
            Corner::URB, Corner::ULF,
            Corner::DRF, Corner::DLB,
        ];
        let corner_positions = PAIRED_CORNERS.map(|pos| *cube.get_corner_position(&pos));
        
        let mut positions = Vec::from(PAIRED_CORNERS);
        
        let pair = &corner_positions[0..2];
        let pair_index1 = combination_rank(pair, &positions);
        positions.retain(|pos| !pair.contains(pos));

        let pair = &corner_positions[2..4];
        let pair_index2 = combination_rank(pair, &positions);
        positions.retain(|pos| !pair.contains(pos));

        let pair = &corner_positions[4..6];
        let pair_index3 = combination_rank(pair, &positions);
        
        // (6 choose 2) = 15 and (4 choose 2) = 6
        let corner_pairs_index = (pair_index1 * 15 + pair_index2) * 6 + pair_index3;
        
        let parity = permutation_parity(&corner_positions, &PAIRED_CORNERS);

        const REMAINING_EDGES: [Edge; 8] = [Edge::UF, Edge::DF, Edge::DB, Edge::UB, Edge::UR, Edge::UL, Edge::DL, Edge::DR];
        const M_SLICE_EDGES: [Edge; 4] = [Edge::UF, Edge::DF, Edge::DB, Edge::UB];
        let m_slice_edge_positions = M_SLICE_EDGES.map(|edge| *cube.get_edge_position(&edge));
        let m_slice_edge_index = combination_rank(&m_slice_edge_positions, &REMAINING_EDGES);

        return (corner_pairs_index * 70 + m_slice_edge_index) * 2 + parity as usize
    }
}

pub struct G4;
impl<'a> Stage<'a> for G4 {
    const FILEPATH: &'a str = &"./data/g4.dat";
    const SIZE: usize = 663552;
    const MOVE_POOL: &'a [Move] = &[
        Move::U2,
        Move::L2,
        Move::D2,
        Move::R2,
        Move::F2,
        Move::B2,
    ];

    fn indexer(cube: &Cube) -> usize {
        let tetrad = [Corner::URF, Corner::ULB, Corner::DRB, Corner::DLF];
        let positions = tetrad.map(|corner| *cube.get_corner_position(&corner));
        let tetrad_index = permutation_rank(&positions, &tetrad);
    
        let tetrad = [Corner::URB, Corner::ULF, Corner::DRF, Corner::DLB];
        let position = cube.get_corner_position(&Corner::URB);
        let urb_index = tetrad.iter()
            .position(|corner| corner == position)
            .unwrap();
    
        let corner_index = tetrad_index * 4 + urb_index;
        
        let slice = [Edge::RF, Edge::RB, Edge::LB, Edge::LF];
        let positions = slice.map(|edge| *cube.get_edge_position(&edge));
        let e_slice_index = permutation_rank(&positions, &slice);
    
        let slice = [Edge::UF, Edge::DF, Edge::DB, Edge::UB];
        let positions = slice.map(|edge| *cube.get_edge_position(&edge));
        let m_slice_index = permutation_rank(&positions, &slice);

        let slice = [Edge::UR, Edge::UL, Edge::DL, Edge::DR];
        let partial_permutation: Vec<usize> = [Edge::UR, Edge::UL].iter()
            .map(|edge| *cube.get_edge_position(&edge))
            .map(|edge| slice.iter().position(|&x| x == edge).unwrap())
            .collect();
        let s_slice_index = combination_rank(&partial_permutation, &[0, 1, 2, 3]) * 2 + (partial_permutation[0] < partial_permutation[1]) as usize;
        
        let edge_index = (s_slice_index * 24 + m_slice_index) * 24 + e_slice_index;
        
        return edge_index * 96 + corner_index
    }
}


/**Computes a unique index for `combination` between all similar sized combinations of 
 * `ordering`. (a combination is an unordered subset). In particular, the computed index
 * is the co-lexicographic rank of the combination, where the lexicographic ordering is
 * defined by `ordering`.
 * 
 * Implementation from: https://computationalcombinatorics.wordpress.com/2012/09/10/ranking-and-unranking-of-combinations-and-permutations/
 * First the combination of items is transformed to an integer combination. Then the rank
 * of this combination is computed by the formula: rank(S) = Σ_{i=0}^{k-1} (S_i choose i+1),
 * where S is the list of integers in increasing order. Justification of the formula is not 
 * very well described at the source. Will have to investigate this at some point.
 */
fn combination_rank<T: PartialEq>(combination: &[T], ordering: &[T]) -> usize {
    let mut combination: Vec<usize> = combination.iter()
        .map(|x| ordering.iter().position(|y| x == y).unwrap())
        .collect();
    combination.sort();
    combination.iter()
        .enumerate()
        .map(|(i, &elem)| binom(elem, i + 1))
        .sum()
}

/// Computes the binomial coefficient (n choose k).
fn binom(n: usize, k: usize) -> usize {
    if k > n {
        0
    } else {
        (0..k).fold(1, |res, i| (res * (n - i)) / (i + 1))
    }
}

/**Lexicographic rank of a permutation. This implementation is O(n) and is taken
 * from Wendy Myrvold, Frank Ruskey, Ranking and unranking permutations in linear time, 
 * Information Processing Letters, Volume 79, Issue 6, 2001, Pages 281-284,
 */
fn permutation_rank_recursive<T: PartialEq>(permutation: &[T], initial: &[T]) -> usize {
    let pi: Vec<usize> = permutation.iter()
        .map(|x| initial.iter().position(|y| x == y).unwrap())
        .collect();
    let mut pi_inv = pi.clone();
    for (i, pi_i) in pi.iter().enumerate() {
        pi_inv[*pi_i] = i;
    };

    return _permutation_rank_recursive(pi.len(), pi, pi_inv)
}

fn _permutation_rank_recursive(n: usize, mut pi: Vec<usize>, mut pi_inv: Vec<usize>) -> usize {
    if n == 0 {
        return 1;
    }
    let s = pi[n - 1];
    pi.swap(n - 1, pi_inv[n - 1]);
    pi_inv.swap(s, n - 1);
    return s + n * _permutation_rank_recursive(n - 1, pi, pi_inv)
}

/// Lexicographic rank of a permutation
fn permutation_rank<T: PartialEq>(permutation: &[T], initial: &[T]) -> usize {
    let pi: Vec<usize> = permutation.iter()
        .map(|x| initial.iter().position(|y| x == y).unwrap())
        .collect();
    
    let n = pi.len();
    let mut digits: Vec<usize> = (0..n).collect();
    let mut factorial: usize = (2..n).product();
    let mut index = 0;
    for i in 0..n-1 {
        let q = digits.iter()
            .position(|x| x == &pi[i])
            .unwrap();
        index += factorial * q;
        digits.remove(q);
        factorial /= n - 1 - i;
    }
    return index;
}

fn permutation_parity<T: PartialEq>(permutation: &[T], initial: &[T]) -> bool {
    let permutation: Vec<usize> = permutation.iter()
        .map(|x| initial.iter().position(|y| x == y).unwrap())
        .collect();

    let mut parity = false;
    for i in 0..permutation.len() {
        for j in i+1..permutation.len() {
            parity ^= permutation[i] > permutation[j];
        }
    }
    return parity
}

fn paired_combination_index(corner_positions: &Vec<u8>, mut positions: Vec<u8>) -> usize {
    let pair = &corner_positions[0..2];
    let pair_index1 = combination_rank(pair, &positions);
    positions.retain(|pos| !pair.contains(pos));

    let pair = &corner_positions[2..4];
    let pair_index2 = combination_rank(pair, &positions);
    
    let corner_pairs_index = pair_index1 * binom(4, 2) + pair_index2;
    return corner_pairs_index
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test1() {
        let mut res = vec![vec![0;6];90];
        for i in 0..6 {
            for j in i+1..6 {
                for k in 0..6 {
                    for l in k+1..6 {
                        if [i, j].contains(&k) || [i, j].contains(&l) {continue;}
                        for m in 0..6 {
                            for n in m+1..6 {
                                if [i, j, k, l].contains(&m) || [i, j, k, l].contains(&n) {continue;}
                                let list = vec![i, j, k, l, m, n];
                                let index = paired_combination_index(&list, (0..6).collect());
                                assert_eq!(res[index], vec![0;6]);
                                // dbg!(&list, &index);
                                res[index] = list;
                            }
                        }
                    }
                }
            }
        }
        // dbg!(&res[90 - 6*1 - 1..90 - 6*0]);
        assert!(!res.contains(&vec![0;6]));

        assert_eq!(permutation_parity(&[1,2,4,3], &[2,1,3,4]), false);
    }
}