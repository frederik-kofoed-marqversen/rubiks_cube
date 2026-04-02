//! Thistlethwaite's Algorithm - Stage Definitions and Indexers
//!
//! This module implements the four stages of Thistlethwaite's algorithm for solving
//! the Rubik's Cube. The algorithm progressively reduces the cube through a chain of
//! nested subgroups until reaching the solved state.
//!
//! # Algorithm Overview
//!
//! Thistlethwaite's algorithm works by solving the cube in stages, where each stage
//! reduces cubes from one subgroup to the next using only moves that preserve the
//! previous subgroup's constraints:
//!
//! ```text
//! Full Group (G0)  →  G1  →  G2  →  G3  →  G4 (Solved)
//!                 Stage1  Stage2  Stage3  Stage4
//! ```
//!
//! Each stage Gi:
//! - **Operates on**: Cubes already in subgroup Gi-1 (prerequisite/invariant)
//! - **Uses moves**: Only those that preserve Gi-1 (maintain previous constraints)
//! - **Achieves**: Reduction to subgroup Gi (adds new constraints)
//! - **State space**: All equivalence classes in Gi-1 that aren't yet in Gi
//!
//! For example, Stage G2 takes cubes that are already in G1 (edges oriented) and
//! uses only moves that preserve G1, progressively reducing them to G2 (corners
//! oriented, E-slice placed).
//!
//! # Stage Progression
//!
//! - **Stage G1**: Orient all edges → subgroup where edges can't flip
//! - **Stage G2**: Orient corners & place E-slice → subgroup with oriented pieces
//! - **Stage G3**: Separate edge slices & fix corner structure → highly constrained subgroup
//! - **Stage G4**: Solve completely → identity (solved cube)

use super::cube::{Cube, Move, Edge, Corner, EDGES, CORNERS};

pub trait Stage {
    /// Move pool for this stage: moves that preserve the previous subgroup.
    /// These are the only legal moves when reducing from Gi-1 to Gi.
    const MOVE_POOL: &'static [Move];
    
    /// Number of equivalence classes in the coset space G_{i-1} / G_i.
    /// 
    /// This is the group index [G_{i-1} : G_i] = |G_{i-1}| / |G_i|, representing how many
    /// distinct "patterns" exist in G_{i-1} when we ignore properties already fixed by G_i.
    /// 
    /// For example, G2::SIZE = 1,082,565 is the number of distinct (corner orientation,
    /// E-slice position) combinations among G1 states. These are the equivalence classes
    /// that distinguish G1 states from each other relative to G2.
    const SIZE: usize;
    
    /// Compute a dense array index for this cube state.
    /// 
    /// Maps cube states to integers in the range [0, SIZE), enabling efficient
    /// storage in dense lookup tables. Each equivalence class gets a unique index.
    fn indexer(cube: &Cube) -> usize;

    fn name() -> &'static str {
        std::any::type_name::<Self>().rsplit("::").next().unwrap()
    }
}

/// **Stage G1: Orient all edges**
/// 
/// Reduces cubes from the full group (G0) to subgroup G1 where all edges are oriented.
/// 
/// **Subgroup G1 definition**: All edges have orientation = 0
/// 
/// **Move pool**: All 18 moves (full group)
///   - No restrictions yet since we're starting from any scrambled state
///   - After reaching G1, only moves that preserve edge orientation can be used
/// 
/// **State space (SIZE = 2,048)**: Equivalence classes of G0 under "same edge orientations"
///   - Each of 12 edges can be flipped: 2^12 possibilities
///   - Parity constraint reduces this to 2^11 = 2,048 valid patterns
///   - These patterns partition all possible scrambles into 2,048 equivalence classes
/// 
/// **Invariant**: None (operates on any cube state)
pub struct G1;
impl Stage for G1 {
    const SIZE: usize = 2048;
    const MOVE_POOL: &'static [Move] = &[
        Move::U, Move::Up, Move::U2,
        Move::L, Move::Lp, Move::L2,
        Move::D, Move::Dp, Move::D2,
        Move::R, Move::Rp, Move::R2,
        Move::F, Move::Fp, Move::F2,
        Move::B, Move::Bp, Move::B2,
    ];

    /// Maps each distinct edge orientation pattern to an integer in [0, 2048).
    /// 
    /// **Algorithm**: Treat edge orientations as a base-2 number, using only the
    /// first 11 edges (the 12th is determined by parity: total flips must be even).
    /// 
    /// **Index formula**: Σ(2^i × orientation[edge_i]) for i in 0..11
    fn indexer(cube: &Cube) -> usize {
        EDGES[..11]
            .iter()
            .enumerate()
            .map(|(i, edge)| (cube.get_edge_orientation(edge) as usize) << i)
            .sum()
    }
}

/// **Stage G2: Orient corners and place E-slice edges**
/// 
/// Reduces cubes from subgroup G1 (edges oriented) to subgroup G2 where corners
/// are also oriented and E-slice edges are in their slice.
/// 
/// **Subgroup G2 definition**:
///   - All edges oriented (inherited from G1)
///   - All corners oriented (new constraint)
///   - E-slice edges in E-slice positions (new constraint)
/// 
/// **Move pool**: {U, D, L, R, F2, B2}
///   - No quarter turns of F or B (these would flip edges, breaking G1)
///   - These moves preserve edge orientation while allowing corner/edge permutation
/// 
/// **State space (SIZE = 1,082,565)**: Equivalence classes of G1 under "same corner
///   orientations and E-slice position"
///   - Corner orientations: 3^7 = 2,187 distinct patterns (using 7 of 8 due to parity)
///   - E-slice positions: C(12,4) = 495 ways to place 4 edges in 12 positions
///   - Total: 2,187 × 495 = 1,082,565 equivalence classes partitioning G1
/// 
/// **Invariant**: All edges must be oriented (G1 constraint)
pub struct G2;
impl Stage for G2 {
    const SIZE: usize = 1082565;
    const MOVE_POOL: &'static [Move] = &[
        Move::U, Move::Up, Move::U2,
        Move::L, Move::Lp, Move::L2,
        Move::D, Move::Dp, Move::D2,
        Move::R, Move::Rp, Move::R2,
        Move::F2,
        Move::B2,
    ];

    /// Maps each G1 state (that's not yet in G2) to an integer in [0, 1,082,565).
    /// 
    /// **Algorithm**: Combines two independent components:
    /// 1. Corner orientation index: base-3 number using first 7 corners (3^7 = 2,187)
    /// 2. E-slice position index: which 4 of 12 positions hold E-slice edges (495)
    /// 
    /// **Index formula**: corner_orientation_index × 495 + e_slice_position_index
    fn indexer(cube: &Cube) -> usize {
        const POWERS_OF_3: [usize; 7] = [1, 3, 9, 27, 81, 243, 729];
        
        // Compute corner orientation index as base-3 number
        let corner_orientations = CORNERS[..7].iter()
            .map(|corner| cube.get_corner_orientation(corner));
        let corner_orientation_index = corner_orientations.enumerate()
            .map(|(i, n)| POWERS_OF_3[i] * n as usize)
            .sum::<usize>();

        // Find which 4 positions currently contain the E-slice edges
        const E_SLICE_EDGES: [Edge; 4] = [Edge::RF, Edge::RB, Edge::LB, Edge::LF];
        let e_slice_edge_positions = E_SLICE_EDGES.map(|edge| *cube.get_edge_position(&edge));
        let e_slice_edges_index = combination_rank(&e_slice_edge_positions, &EDGES);

        let index = corner_orientation_index * 495 + e_slice_edges_index;
        return index
    }
}

/// **Stage G3: Separate edge slices and establish corner tetrads**
/// 
/// Reduces cubes from subgroup G2 (oriented pieces, E-slice placed) to subgroup G3
/// where edges are separated into slices and corners form fixed tetrads.
/// 
/// **Subgroup G3 definition** (Pochmann's variation):
///   - All edges and corners oriented (inherited from G1/G2)
///   - Edges separated: E-slice, M-slice, and S-slice edges stay in their slices
///   - Corners in fixed pairs with even permutation parity
/// 
/// **Move pool**: {U, D, L2, R2, F2, B2}
///   - Only quarter turns of U/D allowed (L, R, F, B must be 180°)
///   - These moves preserve G2 while separating edges into slices
/// 
/// **State space (SIZE = 352,800)**: Equivalence classes of G2 under "same tetrad
///   positions, M-slice position, and parity"
///   - Corner pair placements: C(8,2)×C(6,2)×C(4,2) = 2,520 patterns
///   - M-slice edge positions: C(8,4) = 70 configurations among non-E positions
///   - Permutation parity: 2 states (even/odd)
///   - Total: 2,520 × 70 × 2 = 352,800 equivalence classes partitioning G2
/// 
/// **Invariant**: All edges/corners oriented, E-slice edges in E-slice (G2 constraints)
/// 
/// **Note**: Uses Pochmann's corner pairing instead of traditional tetrads for simpler indexing.
pub struct G3Pochmann;
impl Stage for G3Pochmann {
    const SIZE: usize = 352800;
    const MOVE_POOL: &'static [Move] = &[
        Move::U, Move::Up, Move::U2,
        Move::L2,
        Move::D, Move::Dp, Move::D2,
        Move::R2,
        Move::F2,
        Move::B2,
    ];

    /// Maps each G2 state (not yet in G3) to an integer in [0, 352,800).
    /// 
    /// **Algorithm**: Combines three components:
    /// 1. Corner pair positions: which 2 positions each of 4 pairs occupies (2,520)
    /// 2. M-slice edge positions: which 4 of 8 non-E-slice positions hold them (70)
    /// 3. Permutation parity: even or odd (2)
    /// 
    /// **Index formula**: (corner_pairs × 70 + m_slice_position) × 2 + parity
    fn indexer(cube: &Cube) -> usize {
        // Define 4 corner pairs that must stay together
        // Pair 0: (URF, ULB), Pair 1: (DRB, DLF), Pair 2: (URB, ULF), Pair 3: (DRF, DLB)
        const PAIRED_CORNERS: [Corner; 8] = [
            Corner::URF, Corner::ULB,  // Pair 0
            Corner::DRB, Corner::DLF,  // Pair 1
            Corner::URB, Corner::ULF,  // Pair 2
            Corner::DRF, Corner::DLB,  // Pair 3
        ];
        let corner_positions = PAIRED_CORNERS.map(|pos| *cube.get_corner_position(&pos));
        
        // Track which positions each pair occupies using combination ranks
        let mut positions = Vec::from(PAIRED_CORNERS);
        
        // Compute index for where each pair is located
        // We choose 2 positions from remaining positions for each pair
        let pair = &corner_positions[0..2];
        let pair_index1 = combination_rank(pair, &positions);  // (8 choose 2) = 28
        positions.retain(|pos| !pair.contains(pos));

        let pair = &corner_positions[2..4];
        let pair_index2 = combination_rank(pair, &positions);  // (6 choose 2) = 15
        positions.retain(|pos| !pair.contains(pos));

        let pair = &corner_positions[4..6];
        let pair_index3 = combination_rank(pair, &positions);  // (4 choose 2) = 6
        
        // Combine pair indices: 28 × 15 × 6 = 2,520 total configurations
        let corner_pairs_index = (pair_index1 * 15 + pair_index2) * 6 + pair_index3;
        
        // Track parity: must be even to be solvable in G4
        let parity = permutation_parity(&corner_positions, &PAIRED_CORNERS);

        // Track M-slice edges among the 8 non-E-slice positions
        // E-slice already fixed in G2, S-slice determined once E and M are placed
        const REMAINING_EDGES: [Edge; 8] = [Edge::UF, Edge::DF, Edge::DB, Edge::UB, Edge::UR, Edge::UL, Edge::DL, Edge::DR];
        const M_SLICE_EDGES: [Edge; 4] = [Edge::UF, Edge::DF, Edge::DB, Edge::UB];
        let m_slice_edge_positions = M_SLICE_EDGES.map(|edge| *cube.get_edge_position(&edge));
        let m_slice_edge_index = combination_rank(&m_slice_edge_positions, &REMAINING_EDGES);  // (8 choose 4) = 70

        return (corner_pairs_index * 70 + m_slice_edge_index) * 2 + parity as usize
    }
}

/// **Stage G4: Solve to completion**
/// 
/// Reduces cubes from subgroup G3 (slices separated, tetrads fixed) to the solved state.
/// 
/// **Subgroup G4 definition**: The identity (solved cube)
///   - All pieces in correct positions and orientations
/// 
/// **Move pool**: {U2, D2, L2, R2, F2, B2}
///   - Only 180° turns allowed
///   - These moves preserve G3 (slices stay separate, tetrad structure maintained)
///   - Gradually permute pieces within slices until solved
/// 
/// **State space (SIZE = 663,552)**: Equivalence classes of G3 under "solved"
///   - Corner permutations: 4! × 4 = 96 (one tetrad permuted fully, one position tracked)
///   - Edge permutations within slices: 4! × 4! × 12 = 6,912 (E-slice × M-slice × partial S-slice)
///   - Total: 96 × 6,912 = 663,552 equivalence classes partitioning G3
/// 
/// **Invariant**: All G3 constraints (oriented pieces, separated slices, fixed tetrads)
pub struct G4;
impl Stage for G4 {
    const SIZE: usize = 663552;
    const MOVE_POOL: &'static [Move] = &[
        Move::U2,
        Move::L2,
        Move::D2,
        Move::R2,
        Move::F2,
        Move::B2,
    ];

    /// Maps each G3 state (not yet solved) to an integer in [0, 663,552).
    /// 
    /// **Algorithm**: Combines corner and edge permutation indices:
    /// 1. Corner permutations (96): first tetrad permutation × second tetrad position
    /// 2. Edge permutations (6,912): permutations within each of 3 slices
    /// 
    /// **Index formula**: edge_index × 96 + corner_index
    fn indexer(cube: &Cube) -> usize {
        // Corner index: permutation of first tetrad × position of one corner in second tetrad
        // First tetrad: 4! = 24 permutations
        let tetrad = [Corner::URF, Corner::ULB, Corner::DRB, Corner::DLF];
        let positions = tetrad.map(|corner| *cube.get_corner_position(&corner));
        let tetrad_index = permutation_rank(&positions, &tetrad);  // 0..23
    
        // Second tetrad: just track URB's position (other 3 determined by first tetrad)
        let tetrad = [Corner::URB, Corner::ULF, Corner::DRF, Corner::DLB];
        let position = cube.get_corner_position(&Corner::URB);
        let urb_index = tetrad.iter()
            .position(|corner| corner == position)
            .unwrap();  // 0..3
    
        let corner_index = tetrad_index * 4 + urb_index;  // 0..95  // 0..95
        
        // Edge index: permutations within each of the 3 slices
        // E-slice (middle layer): 4! = 24 permutations
        let slice = [Edge::RF, Edge::RB, Edge::LB, Edge::LF];
        let positions = slice.map(|edge| *cube.get_edge_position(&edge));
        let e_slice_index = permutation_rank(&positions, &slice);  // 0..23
    
        // M-slice (front-back): 4! = 24 permutations
        let slice = [Edge::UF, Edge::DF, Edge::DB, Edge::UB];
        let positions = slice.map(|edge| *cube.get_edge_position(&edge));
        let m_slice_index = permutation_rank(&positions, &slice);  // 0..23

        // S-slice (left-right): partial permutation
        // Track which 2 of 4 positions hold UR and UL, plus their relative order
        let slice = [Edge::UR, Edge::UL, Edge::DL, Edge::DR];
        let partial_permutation: Vec<usize> = [Edge::UR, Edge::UL].iter()
            .map(|edge| *cube.get_edge_position(&edge))
            .map(|edge| slice.iter().position(|&x| x == edge).unwrap())
            .collect();
        // (4 choose 2) = 6 positions, × 2 orderings = 12 configurations
        let s_slice_index = combination_rank(&partial_permutation, &[0, 1, 2, 3]) * 2 + (partial_permutation[0] < partial_permutation[1]) as usize;  // 0..11  // 0..11
        
        // Combine edge indices: 12 × 24 × 24 = 6,912 configurations
        let edge_index = (s_slice_index * 24 + m_slice_index) * 24 + e_slice_index;  // 0..6911
        
        // Final index: 6,912 × 96 = 663,552 total configurations
        return edge_index * 96 + corner_index
    }
}


/// Compute the co-lexicographic rank of a combination (unordered subset).
/// 
/// Given a combination (subset) and an ordering of all elements, this computes a unique
/// index representing which elements are in the subset. This is used to index states like
/// "which 4 positions hold E-slice edges" out of 12 total positions.
/// 
/// # Implementation
/// Uses the formula: rank(S) = Σ(S_i choose i+1) for sorted indices S.
/// See: https://computationalcombinatorics.wordpress.com/2012/09/10/ranking-and-unranking-of-combinations-and-permutations/
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

/// Binomial coefficient "n choose k"
/// Uses iterative multiplication and division to avoid overflow and improve numerical stability.
/// More efficient than computing factorials separately.
fn binom(n: usize, k: usize) -> usize {
    if k > n {
        0
    } else {
        (0..k).fold(1, |res, i| (res * (n - i)) / (i + 1))
    }
}

/// Compute the lexicographic rank of a permutation.
/// 
/// Given a permutation and the initial (solved) ordering, this computes a unique index
/// representing which of the n! possible permutations this is. Used to index states like
/// "which permutation of the 4 E-slice edges" (4! = 24 possibilities).
/// 
/// # Implementation
/// Uses O(n²) factorial number system. More efficient O(n) algorithms exist but
/// require additional bookkeeping. This is fast enough for n ≤ 8.
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

        // Remove the used digit by swapping it to the end of the list
        // digits.remove(q);
        digits.swap(q, n - 1 - i);
        factorial /= n - 1 - i;
    }
    return index;
}

/**Lexicographic rank of a permutation. This implementation is O(n) and is taken
 * from Wendy Myrvold, Frank Ruskey, Ranking and unranking permutations in linear time, 
 * Information Processing Letters, Volume 79, Issue 6, 2001, Pages 281-284,
 */
#[allow(dead_code)]
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

/// Compute the parity (even/odd) of a permutation.
/// Counts inversions (pairs where permutation[i] > permutation[j] for i < j).
/// Parity is odd if inversion count is odd. O(n²) but simple and correct.
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
