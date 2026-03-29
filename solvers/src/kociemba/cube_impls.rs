// This file will contain Kociemba-specific Cube implementations
// Currently commented out pending Cube structure finalization

/* 
use super::cube::Cube;

impl Cube {
    pub fn corner_parity(&self) -> u32 {
        permutation_parity(&self.corner_permutation) as u32
    }

    pub fn edge_parity(&self) -> u32 {
        permutation_parity(&self.edge_permutation) as u32
    }
    
    pub fn corner_orientation_index(&self) -> u32 {
        /*Compute corner orientation index < 3^7 = 2187 */
        self.corner_orientation
            .iter()
            .enumerate()
            .fold(0, |res, (i, orientation)| {
                res + 3_u32.pow(i as u32) * orientation
            })
    }

    pub fn edge_orientation_index(&self) -> u32 {
        /*Compute edge orientation index < 2^11 = 2048 */
        self.edge_orientation
            .iter()
            .enumerate()
            .fold(0, |res, (i, orientation)| {
                res + 2_u32.pow(i as u32) * orientation
            })
    }
}

fn permutation_parity<T: Ord>(permutation: &[T]) -> bool {
    let mut parity = false;
    for i in 0..permutation.len() {
        for j in i + 1..permutation.len() {
            parity ^= permutation[i] > permutation[j];
        }
    }
    return parity;
}
*/