use crate::rng::{Rng, random_permutation};
use crate::math::{permutation_parity, compose_permutations};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    UR, UB, UL, UF, // top layer
    RF, RB, LB, LF, // middle layer (E-slice)
    DR, DB, DL, DF, // bottom layer
}

pub const EDGES: [Edge; 12] = [
    Edge::UR, Edge::UB, Edge::UL, Edge::UF,
    Edge::RF, Edge::RB, Edge::LB, Edge::LF,
    Edge::DR, Edge::DB, Edge::DL, Edge::DF,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Corner {
    URF, URB, ULB, ULF, // top layer
    DRB, DRF, DLF, DLB, // bottom layer
}

pub const CORNERS: [Corner; 8] = [
    Corner::URF, Corner::URB, Corner::ULB, Corner::ULF,
    Corner::DRB, Corner::DRF, Corner::DLF, Corner::DLB,
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Move {
    U, Up, U2,
    L, Lp, L2,
    D, Dp, D2,
    R, Rp, R2,
    F, Fp, F2,
    B, Bp, B2,
}

pub const MOVES: [Move; 18] = [
    Move::U, Move::Up, Move::U2,
    Move::L, Move::Lp, Move::L2,
    Move::D, Move::Dp, Move::D2,
    Move::R, Move::Rp, Move::R2,
    Move::F, Move::Fp, Move::F2,
    Move::B, Move::Bp, Move::B2,
];

impl Move {
    pub fn inverse(&self) -> Self {
        match self {
            Move::U => Move::Up,
            Move::Up => Move::U,
            Move::U2 => Move::U2,
            Move::L => Move::Lp,
            Move::Lp => Move::L,
            Move::L2 => Move::L2,
            Move::D => Move::Dp,
            Move::Dp => Move::D,
            Move::D2 => Move::D2,
            Move::R => Move::Rp,
            Move::Rp => Move::R,
            Move::R2 => Move::R2,
            Move::F => Move::Fp,
            Move::Fp => Move::F,
            Move::F2 => Move::F2,
            Move::B => Move::Bp,
            Move::Bp => Move::B,
            Move::B2 => Move::B2,
        }
    }

    pub fn face(&self) -> Face {
        match self {
            Move::U | Move::Up | Move::U2 => Face::U,
            Move::L | Move::Lp | Move::L2 => Face::L,
            Move::D | Move::Dp | Move::D2 => Face::D,
            Move::R | Move::Rp | Move::R2 => Face::R,
            Move::F | Move::Fp | Move::F2 => Face::F,
            Move::B | Move::Bp | Move::B2 => Face::B,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Face {
    U, D, L, R, F, B,
}

impl Face {
    pub fn opposite(&self) -> Self {
        match self {
            Face::U => Face::D,
            Face::D => Face::U,
            Face::L => Face::R,
            Face::R => Face::L,
            Face::F => Face::B,
            Face::B => Face::F,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Cube {
    // Position-indexed arrays:
    // edge_perm[pos] = ID (0-11) of the piece currently at position pos
    // edge_orient[pos] = orientation of the piece currently at position pos
    edge_perm: [usize; 12],
    edge_orient: [u32; 12],
    corner_perm: [usize; 8],
    corner_orient: [u32; 8],
}

#[inline]
fn flip_edge(orientation: u32) -> u32 {
    orientation ^ 1
}

#[inline]
fn rotate_corner(orientation: u32, amount: u32) -> u32 {
    (orientation + amount) % 3
}

// def inv_cubie_cube(self, d):
//         """Store the inverse of this cubie cube in d."""
//         for e in Ed:
//             d.ep[self.ep[e]] = e
//         for e in Ed:
//             d.eo[e] = self.eo[d.ep[e]]

//         for c in Co:
//             d.cp[self.cp[c]] = c
//         for c in Co:
//             ori = self.co[d.cp[c]]
//             if ori >= 3:
//                 d.co[c] = ori
//             else:
//                 d.co[c] = -ori
//                 if d.co[c] < 0:
//                     d.co[c] += 3

//     def symmetries(self):
//         """Generate a list of the symmetries and antisymmetries of the cubie cube."""
//         from twophase.symmetries import symCube, inv_idx  # not nice here but else we have circular imports
//         s = []
//         d = CubieCube()
//         for j in range(N_SYM):
//             c = CubieCube(symCube[j].cp, symCube[j].co, symCube[j].ep, symCube[j].eo)
//             c.multiply(self)
//             c.multiply(symCube[inv_idx[j]])
//             if self == c:
//                 s.append(j)
//             c.inv_cubie_cube(d)
//             if self == d:  # then we have antisymmetry
//                 s.append(j + N_SYM)
//         return s

impl Cube {
    pub fn new_random(rng: &mut Rng) -> Self {
        let mut edge_permutation: [usize; 12] = random_permutation(rng);
        let corner_permutation: [usize; 8] = random_permutation(rng);
        let edge_parity = permutation_parity(&edge_permutation);
        let corner_parity = permutation_parity(&corner_permutation);
        if edge_parity != corner_parity {
            edge_permutation.swap(0, 1);
        }

        let mut edge_orientations = (0..11).map(|_| rng.u32() % 2).collect::<Vec<_>>();
        let mut corner_orientations = (0..7).map(|_| rng.u32() % 3).collect::<Vec<_>>();
        let edge_parity = edge_orientations.iter().sum::<u32>() % 2;
        let corner_parity = corner_orientations.iter().sum::<u32>() % 3;
        edge_orientations.push((2 - edge_parity) % 2);
        corner_orientations.push((3 - corner_parity) % 3);

        let cube = Cube {
            edge_perm: edge_permutation,
            edge_orient: edge_orientations.try_into().unwrap(),
            corner_perm: corner_permutation,
            corner_orient: corner_orientations.try_into().unwrap(),
        };

        assert!(cube.is_valid(), "Generated an invalid cube state");
        cube
    }
    
    pub fn new_solved() -> Self {
        Cube{
            edge_perm: [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
            edge_orient: [0; 12],
            corner_perm: [0, 1, 2, 3, 4, 5, 6, 7],
            corner_orient: [0; 8],
        }
    }

    /// Create a cube from raw permutation and orientation arrays.
    /// This is pub(crate) to allow the symmetries module to construct cubes
    /// while keeping the internal representation hidden from external users.
    pub(crate) const fn from_arrays(
        edge_perm: [usize; 12],
        edge_orient: [u32; 12],
        corner_perm: [usize; 8],
        corner_orient: [u32; 8],
    ) -> Self {
        Cube {
            edge_perm,
            edge_orient,
            corner_perm,
            corner_orient,
        }
    }

    #[inline]
    pub fn is_solved(&self) -> bool {
        return *self == Cube::new_solved();
    }

    /// Multiply (compose) two cubes: self ∘ other
    /// 
    /// This applies the transformation represented by `other` first, then `self`.
    /// In group theory notation: (self ∘ other)(x) = self(other(x))
    /// 
    /// Example: If `other` scrambles the cube and `self` is a symmetry,
    /// the result is the scrambled cube after applying the symmetry.
    pub fn multiply(&self, other: &Cube) -> Cube {
        // Compose permutations
        let edge_perm = compose_permutations(&self.edge_perm, &other.edge_perm);
        let corner_perm = compose_permutations(&self.corner_perm, &other.corner_perm);
        
        // Compose edge orientations: add mod 2
        // When composing, orientations add: result[i] = (self.orient[i] + other.orient[perm[i]]) % 2
        let mut edge_orient = [0u32; 12];
        for i in 0..12 {
            edge_orient[i] = (self.edge_orient[i] + other.edge_orient[other.edge_perm[i]]) % 2;
        }
        
        // Compose corner orientations: add mod 3
        let mut corner_orient = [0u32; 8];
        for i in 0..8 {
            corner_orient[i] = (self.corner_orient[i] + other.corner_orient[other.corner_perm[i]]) % 3;
        }
        
        Cube {
            edge_perm,
            edge_orient,
            corner_perm,
            corner_orient,
        }
    }

    pub fn is_valid(&self) -> bool {
        // Check corner orientation parity (sum must be 0 mod 3)
        if self.corner_orient.iter().sum::<u32>() % 3 != 0 {
            return false;
        }
        // Check edge orientation parity (sum must be 0 mod 2)
        if self.edge_orient.iter().sum::<u32>() % 2 != 0 {
            return false;
        }
        // Check that corner and edge permutation parities match
        if permutation_parity(&self.corner_perm) != permutation_parity(&self.edge_perm) {
            return false;
        }
        true
    }

    // Getters and setters
    #[inline]
    pub fn get_edge_orientation(&self, pos: Edge) -> u32 {
        self.edge_orient[pos as usize]
    }

    #[inline]
    pub fn get_edge_type(&self, pos: Edge) -> Edge {
        EDGES[self.edge_perm[pos as usize]]
    }

    #[inline]
    pub fn get_edge_position(&self, edge: Edge) -> Edge {
        let edge_id = edge as usize;
        self.edge_perm.iter().position(|&id| id == edge_id).map(|pos| EDGES[pos]).unwrap()
    }

    #[inline]
    pub fn get_corner_orientation(&self, pos: Corner) -> u32 {
        self.corner_orient[pos as usize]
    }

    #[inline]
    pub fn get_corner_type(&self, pos: Corner) -> Corner {
        CORNERS[self.corner_perm[pos as usize]]
    }

    #[inline]
    pub fn get_corner_position(&self, corner: Corner) -> Corner {
        let corner_id = corner as usize;
        self.corner_perm.iter().position(|&id| id == corner_id).map(|pos| CORNERS[pos]).unwrap()
    }

    #[inline]
    pub fn set_edge_orientation(&mut self, pos: Edge, orientation: u32) {
        self.edge_orient[pos as usize] = orientation;
    }

    #[inline]
    pub fn set_corner_orientation(&mut self, pos: Corner, orientation: u32) {
        self.corner_orient[pos as usize] = orientation;
    }

    #[inline]
    pub fn set_edge_type(&mut self, pos: Edge, edge_type: Edge) {
        self.edge_perm[pos as usize] = edge_type as usize;
    }

    #[inline]
    pub fn set_corner_type(&mut self, pos: Corner, corner_type: Corner) {
        self.corner_perm[pos as usize] = corner_type as usize;
    }

    // Internal helper functions for applying moves
    #[inline]
    fn swap_edges(&mut self, pos1: Edge, pos2: Edge) {
        self.edge_perm.swap(pos1 as usize, pos2 as usize);
        self.edge_orient.swap(pos1 as usize, pos2 as usize);
    }

    #[inline]
    fn swap_corners(&mut self, pos1: Corner, pos2: Corner) {
        self.corner_perm.swap(pos1 as usize, pos2 as usize);
        self.corner_orient.swap(pos1 as usize, pos2 as usize);
    }

    fn u(&mut self) -> &mut Self {
        self.swap_edges(Edge::UR, Edge::UB);
        self.swap_edges(Edge::UB, Edge::UL);
        self.swap_edges(Edge::UL, Edge::UF);
        
        self.swap_corners(Corner::URF, Corner::URB);
        self.swap_corners(Corner::URB, Corner::ULB);
        self.swap_corners(Corner::ULB, Corner::ULF);
        
        self
    }

    fn d(&mut self) -> &mut Self {
        self.swap_edges(Edge::DR, Edge::DF);
        self.swap_edges(Edge::DF, Edge::DL);
        self.swap_edges(Edge::DL, Edge::DB);
        
        self.swap_corners(Corner::DRF, Corner::DLF);
        self.swap_corners(Corner::DLF, Corner::DLB);
        self.swap_corners(Corner::DLB, Corner::DRB);
        
        self
    }

    fn r(&mut self) -> &mut Self {
        self.swap_edges(Edge::UR, Edge::RF);
        self.swap_edges(Edge::RF, Edge::DR);
        self.swap_edges(Edge::DR, Edge::RB);

        self.swap_corners(Corner::URF, Corner::DRF);
        self.swap_corners(Corner::DRF, Corner::DRB);
        self.swap_corners(Corner::DRB, Corner::URB);

        self.corner_orient[Corner::URF as usize] = rotate_corner(self.corner_orient[Corner::URF as usize], 1);
        self.corner_orient[Corner::DRF as usize] = rotate_corner(self.corner_orient[Corner::DRF as usize], 2);
        self.corner_orient[Corner::DRB as usize] = rotate_corner(self.corner_orient[Corner::DRB as usize], 1);
        self.corner_orient[Corner::URB as usize] = rotate_corner(self.corner_orient[Corner::URB as usize], 2);

        self
    }

    fn l(&mut self) -> &mut Self {
        self.swap_edges(Edge::UL, Edge::LB);
        self.swap_edges(Edge::LB, Edge::DL);
        self.swap_edges(Edge::DL, Edge::LF);

        self.swap_corners(Corner::ULF, Corner::ULB);
        self.swap_corners(Corner::ULB, Corner::DLB);
        self.swap_corners(Corner::DLB, Corner::DLF);

        self.corner_orient[Corner::ULF as usize] = rotate_corner(self.corner_orient[Corner::ULF as usize], 2);
        self.corner_orient[Corner::DLF as usize] = rotate_corner(self.corner_orient[Corner::DLF as usize], 1);
        self.corner_orient[Corner::DLB as usize] = rotate_corner(self.corner_orient[Corner::DLB as usize], 2);
        self.corner_orient[Corner::ULB as usize] = rotate_corner(self.corner_orient[Corner::ULB as usize], 1);

        self
    }

    fn f(&mut self) -> &mut Self {
        self.swap_edges(Edge::UF, Edge::LF);
        self.swap_edges(Edge::LF, Edge::DF);
        self.swap_edges(Edge::DF, Edge::RF);

        self.swap_corners(Corner::URF, Corner::ULF);
        self.swap_corners(Corner::ULF, Corner::DLF);
        self.swap_corners(Corner::DLF, Corner::DRF);

        self.corner_orient[Corner::URF as usize] = rotate_corner(self.corner_orient[Corner::URF as usize], 2);
        self.corner_orient[Corner::ULF as usize] = rotate_corner(self.corner_orient[Corner::ULF as usize], 1);
        self.corner_orient[Corner::DLF as usize] = rotate_corner(self.corner_orient[Corner::DLF as usize], 2);
        self.corner_orient[Corner::DRF as usize] = rotate_corner(self.corner_orient[Corner::DRF as usize], 1);

        self.edge_orient[Edge::UF as usize] = flip_edge(self.edge_orient[Edge::UF as usize]);
        self.edge_orient[Edge::LF as usize] = flip_edge(self.edge_orient[Edge::LF as usize]);
        self.edge_orient[Edge::DF as usize] = flip_edge(self.edge_orient[Edge::DF as usize]);
        self.edge_orient[Edge::RF as usize] = flip_edge(self.edge_orient[Edge::RF as usize]);
        
        self
    }

    fn b(&mut self) -> &mut Self {
        self.swap_edges(Edge::UB, Edge::RB);
        self.swap_edges(Edge::RB, Edge::DB);
        self.swap_edges(Edge::DB, Edge::LB);

        self.swap_corners(Corner::URB, Corner::DRB);
        self.swap_corners(Corner::DRB, Corner::DLB);
        self.swap_corners(Corner::DLB, Corner::ULB);

        self.corner_orient[Corner::URB as usize] = rotate_corner(self.corner_orient[Corner::URB as usize], 1);
        self.corner_orient[Corner::ULB as usize] = rotate_corner(self.corner_orient[Corner::ULB as usize], 2);
        self.corner_orient[Corner::DLB as usize] = rotate_corner(self.corner_orient[Corner::DLB as usize], 1);
        self.corner_orient[Corner::DRB as usize] = rotate_corner(self.corner_orient[Corner::DRB as usize], 2);

        self.edge_orient[Edge::UB as usize] = flip_edge(self.edge_orient[Edge::UB as usize]);
        self.edge_orient[Edge::LB as usize] = flip_edge(self.edge_orient[Edge::LB as usize]);
        self.edge_orient[Edge::DB as usize] = flip_edge(self.edge_orient[Edge::DB as usize]);
        self.edge_orient[Edge::RB as usize] = flip_edge(self.edge_orient[Edge::RB as usize]);
        
        self
    }
}

pub trait Moveable {
    fn turn(&mut self, mv: Move) -> &mut Self;

    fn apply_moves(&mut self, moves: &[Move]) -> &mut Self {
        for &turn in moves {
            self.turn(turn);
        }
        self
    }
}

impl Moveable for Cube {
    fn turn(&mut self, turn: Move) -> &mut Self {
        match turn {
            Move::U  => self.u(),
            Move::U2 => self.u().u(),
            Move::Up => self.u().u().u(),
            Move::D  => self.d(),
            Move::D2 => self.d().d(),
            Move::Dp => self.d().d().d(),
            Move::R  => self.r(),
            Move::R2 => self.r().r(),
            Move::Rp => self.r().r().r(),
            Move::L  => self.l(),
            Move::L2 => self.l().l(),
            Move::Lp => self.l().l().l(),
            Move::F  => self.f(),
            Move::F2 => self.f().f(),
            Move::Fp => self.f().f().f(),
            Move::B  => self.b(),
            Move::B2 => self.b().b(),
            Move::Bp => self.b().b().b(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_solved() {
        let cube = Cube::new_solved();
        assert!(cube.is_solved());
        assert!(cube.is_valid());
    }

    #[test]
    fn test_moves_preserve_validity() {
        let mut cube = Cube::new_solved();
        let moves = [Move::R, Move::U, Move::Rp, Move::Up, Move::R, Move::U2, Move::Rp];
        cube.apply_moves(&moves);
        assert!(cube.is_valid());
    }

    #[test]
    fn test_move_inverses() {
        let mut cube = Cube::new_solved();
        
        for mv in &[Move::U, Move::D, Move::L, Move::R, Move::F, Move::B] {
            cube.turn(*mv);
            cube.turn(mv.inverse());
            assert!(cube.is_solved(), "Move {:?} and inverse should return to solved", mv);
        }
    }

    #[test]
    fn test_double_moves() {
        let mut cube1 = Cube::new_solved();
        let mut cube2 = Cube::new_solved();
        
        let moves = vec![Move::R2, Move::U2].repeat(3);
        cube1.apply_moves(&moves);

        cube2.swap_edges(Edge::UF, Edge::UB);
        cube2.swap_edges(Edge::RF, Edge::RB);

        assert_eq!(cube1, cube2);

        let moves = vec![Move::L2, Move::F2].repeat(3);
        cube1.apply_moves(&moves);

        cube2.swap_edges(Edge::UF, Edge::DF);
        cube2.swap_edges(Edge::UL, Edge::DL);

        assert_eq!(cube1, cube2);

        let moves = vec![Move::D2, Move::B2].repeat(3);
        cube1.apply_moves(&moves);

        cube2.swap_edges(Edge::RB, Edge::LB);
        cube2.swap_edges(Edge::DR, Edge::DL);

        assert_eq!(cube1, cube2);
    }

    #[test]
    fn test_corner_commutator() {
        // R' U L U' R U L' U' is a 3-cycle of corners with twists
        let moves = [Move::Rp, Move::U, Move::L, Move::Up, Move::R, Move::U, Move::Lp, Move::Up];
        
        let mut cube = Cube::new_solved();
        cube.apply_moves(&moves);
        
        assert!(cube.is_valid());
        
        // Apply it 3 times should return to solved
        cube.apply_moves(&moves);
        cube.apply_moves(&moves);
        assert!(cube.is_solved());
    }

    #[test]
    fn test_random_cube_validity() {
        let mut rng = Rng::new();
        for _ in 0..100 {
            let cube = Cube::new_random(&mut rng);
            assert!(cube.is_valid(), "Random cube should be valid");
        }
    }
    
    #[test]
    fn test_getter_setter_consistency() {
        let mut cube = Cube::new_solved();
        
        // Test edge getters/setters
        cube.set_edge_type(Edge::UR, Edge::DF);
        assert_eq!(cube.get_edge_type(Edge::UR), Edge::DF);
        
        cube.set_edge_orientation(Edge::UB, 1);
        assert_eq!(cube.get_edge_orientation(Edge::UB), 1);
        
        // Test corner getters/setters
        cube.set_corner_type(Corner::URF, Corner::DLB);
        assert_eq!(cube.get_corner_type(Corner::URF), Corner::DLB);
        
        cube.set_corner_orientation(Corner::URB, 2);
        assert_eq!(cube.get_corner_orientation(Corner::URB), 2);
    }
}