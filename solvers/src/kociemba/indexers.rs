use super::cube::{Corner, Cube, Edge, CORNERS, EDGES};

// Edge orientation in [0, 2048) - 2^11 possible orientations
pub fn edge_orientation_index(cube: &Cube) -> usize {
    let mut index = 0;
    for &pos in EDGES[..11].iter() {
        index <<= 1;
        index |= cube.get_edge_orientation(pos) as usize;
    }
    return index;
}

// Corner orientation in [0, 2187) - 3^7 possible orientations
pub fn corner_orientation_index(cube: &Cube) -> usize {
    let mut index = 0;
    for &pos in CORNERS[..7].iter() {
        index *= 3;
        index += cube.get_corner_orientation(pos) as usize;
    }
    return index;
}

// Combination rank of the 4 edges in the E slice, in [0, 495) - (12 choose 4) possible combinations
pub fn ud_slice_index(cube: &Cube) -> usize {
    const E_SLICE: [Edge; 4] = [Edge::RF, Edge::RB, Edge::LB, Edge::LF];

    let mut index = 0;
    let mut r = 4;

    for (i, &pos) in EDGES.iter().enumerate().rev() {
        if E_SLICE.contains(&cube.get_edge_type(pos)) {
            r -= 1;
        } else if r > 0 {
            // TODO: Precompute binomial coefficients - const BINOM: [[usize; 5]; 12] = ...
            index += binom(i, r);
        }
    }

    return index;
}

fn binom(n: usize, k: usize) -> usize {
    if k > n {
        0
    } else {
        (0..k).fold(1, |res, i| (res * (n - i)) / (i + 1))
    }
}

fn factorial(n: usize) -> usize {
    (2..=n).product()
}

fn phase1_coordinate(cube: &Cube) -> usize {
    let edge_orient = edge_orientation_index(cube);
    let corner_orient = corner_orientation_index(cube);
    let ud_slice = ud_slice_index(cube);

    // Combine the three coordinates into a single index
    // We can use the fact that edge_orient is in [0, 2048), corner_orient in [0, 2187), and ud_slice in [0, 495]
    return (edge_orient * 2187 + corner_orient) * 495 + ud_slice;
}