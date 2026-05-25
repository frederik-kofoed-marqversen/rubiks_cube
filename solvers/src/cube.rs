use crate::rng::{Rng, random_permutation};
use crate::math::{permutation_parity, permutation_inverse};

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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Cubie<T> {
    piece_type: T,
    orientation: u32,
}

impl Cubie<Edge> {
    #[inline]
    fn flip(&mut self) {
        // Addition mod 2
        self.orientation ^= 1;
    }
}

impl Cubie<Corner> {
    #[inline]
    fn rotate(&mut self, amount: u32) {
        // Addition mod 3
        const MOD3: [u32; 5] = [0, 1, 2, 0, 1];
        self.orientation = MOD3[(self.orientation + amount) as usize];
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Cube {
    edges: [Cubie<Edge>; 12],
    corners: [Cubie<Corner>; 8],
    edge_pos: [Edge; 12],
    corner_pos: [Corner; 8],
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

        let edge_pos = permutation_inverse(&edge_permutation);
        let corner_pos = permutation_inverse(&corner_permutation);

        let cube = Cube {
            edges: edge_permutation.map(|i| Cubie{piece_type: EDGES[i], orientation: edge_orientations[i]}),
            corners: corner_permutation.map(|i| Cubie{piece_type: CORNERS[i], orientation: corner_orientations[i]}),
            edge_pos: edge_pos.map(|i| EDGES[i]),
            corner_pos: corner_pos.map(|i| CORNERS[i]),
        };

        assert!(cube.is_valid(), "Generated an invalid cube state");
        cube
    }
    
    pub fn new_solved() -> Self {
        Cube{
            edges: EDGES.map(|edge| Cubie{piece_type: edge, orientation: 0}),
            corners: CORNERS.map(|corner| Cubie{piece_type: corner, orientation: 0}),
            edge_pos: EDGES,
            corner_pos: CORNERS,
        }
    }

    #[inline]
    pub fn is_solved(&self) -> bool {
        return *self == Cube::new_solved();
    }

    pub fn is_valid(&self) -> bool {
        if self.corner_orientation_parity() != 0 {
            return false;
        }
        if self.edge_orientation_parity() != 0 {
            return false;
        }
        if self.corner_permutation_parity() != self.edge_permutation_parity() {
            return false;
        }
        true
    }

    #[inline]
    pub fn corner_permutation_parity(&self) -> usize {
        permutation_parity(&self.corner_pos.map(|c| c as usize))
    }

    #[inline]
    pub fn edge_permutation_parity(&self) -> usize {
        permutation_parity(&self.edge_pos.map(|e| e as usize))
    }

    #[inline]
    pub fn corner_orientation_parity(&self) -> u32 {
        self.corners.iter().map(|c| c.orientation).sum::<u32>() % 3
    }

    #[inline]
    pub fn edge_orientation_parity(&self) -> u32 {
        self.edges.iter().map(|e| e.orientation).sum::<u32>() % 2
    }

    // Getters and setters
    #[inline]
    pub fn get_edge_orientation(&self, pos: Edge) -> u32 {
        self.edges[pos as usize].orientation
    }

    #[inline]
    pub fn get_edge_type(&self, pos: Edge) -> Edge {
        self.edges[pos as usize].piece_type
    }

    #[inline]
    pub fn get_edge_position(&self, edge: Edge) -> Edge {
        self.edge_pos[edge as usize]
    }

    #[inline]
    pub fn get_corner_orientation(&self, pos: Corner) -> u32 {
        self.corners[pos as usize].orientation
    }

    #[inline]
    pub fn get_corner_type(&self, pos: Corner) -> Corner {
        self.corners[pos as usize].piece_type
    }

    #[inline]
    pub fn get_corner_position(&self, corner: Corner) -> Corner {
        self.corner_pos[corner as usize]
    }

    #[inline]
    pub fn set_edge_orientation(&mut self, pos: Edge, orientation: u32) {
        self.edges[pos as usize].orientation = orientation;
    }

    #[inline]
    pub fn set_corner_orientation(&mut self, pos: Corner, orientation: u32) {
        self.corners[pos as usize].orientation = orientation;
    }

    #[inline]
    pub fn set_edge_type(&mut self, pos: Edge, edge_type: Edge) {
        self.edges[pos as usize].piece_type = edge_type;
        self.edge_pos[edge_type as usize] = pos;
    }

    #[inline]
    pub fn set_corner_type(&mut self, pos: Corner, corner_type: Corner) {
        self.corners[pos as usize].piece_type = corner_type;
        self.corner_pos[corner_type as usize] = pos;
    }

    // Internal functions for applying moves
    #[inline]
    fn swap_edges(&mut self, pos1: Edge, pos2: Edge) {
        self.edges.swap(pos1 as usize, pos2 as usize);
        self.edge_pos.swap(self.edges[pos1 as usize].piece_type as usize, self.edges[pos2 as usize].piece_type as usize);
    }

    #[inline]
    fn swap_corners(&mut self, pos1: Corner, pos2: Corner) {
        self.corners.swap(pos1 as usize, pos2 as usize);
        self.corner_pos.swap(self.corners[pos1 as usize].piece_type as usize, self.corners[pos2 as usize].piece_type as usize);
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

        self.corners[Corner::URF as usize].rotate(1);
        self.corners[Corner::DRF as usize].rotate(2);
        self.corners[Corner::DRB as usize].rotate(1);
        self.corners[Corner::URB as usize].rotate(2);

        self
    }

    fn l(&mut self) -> &mut Self {
        self.swap_edges(Edge::UL, Edge::LB);
        self.swap_edges(Edge::LB, Edge::DL);
        self.swap_edges(Edge::DL, Edge::LF);

        self.swap_corners(Corner::ULF, Corner::ULB);
        self.swap_corners(Corner::ULB, Corner::DLB);
        self.swap_corners(Corner::DLB, Corner::DLF);

        self.corners[Corner::ULF as usize].rotate(2);
        self.corners[Corner::DLF as usize].rotate(1);
        self.corners[Corner::DLB as usize].rotate(2);
        self.corners[Corner::ULB as usize].rotate(1);

        self
    }

    fn f(&mut self) -> &mut Self {
        self.swap_edges(Edge::UF, Edge::LF);
        self.swap_edges(Edge::LF, Edge::DF);
        self.swap_edges(Edge::DF, Edge::RF);

        self.swap_corners(Corner::URF, Corner::ULF);
        self.swap_corners(Corner::ULF, Corner::DLF);
        self.swap_corners(Corner::DLF, Corner::DRF);

        self.corners[Corner::URF as usize].rotate(2);
        self.corners[Corner::ULF as usize].rotate(1);
        self.corners[Corner::DLF as usize].rotate(2);
        self.corners[Corner::DRF as usize].rotate(1);

        self.edges[Edge::UF as usize].flip();
        self.edges[Edge::LF as usize].flip();
        self.edges[Edge::DF as usize].flip();
        self.edges[Edge::RF as usize].flip();
        
        self
    }

    fn b(&mut self) -> &mut Self {
        self.swap_edges(Edge::UB, Edge::RB);
        self.swap_edges(Edge::RB, Edge::DB);
        self.swap_edges(Edge::DB, Edge::LB);

        self.swap_corners(Corner::URB, Corner::DRB);
        self.swap_corners(Corner::DRB, Corner::DLB);
        self.swap_corners(Corner::DLB, Corner::ULB);

        self.corners[Corner::URB as usize].rotate(1);
        self.corners[Corner::ULB as usize].rotate(2);
        self.corners[Corner::DLB as usize].rotate(1);
        self.corners[Corner::DRB as usize].rotate(2);

        self.edges[Edge::UB as usize].flip();
        self.edges[Edge::LB as usize].flip();
        self.edges[Edge::DB as usize].flip();
        self.edges[Edge::RB as usize].flip();
        
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
    fn r_l_u_moves() {
        // Short algorithm that affects only few pieces in the end.
        let moves = [Move::Rp, Move::U, Move::L, Move::Up, Move::R, Move::U, Move::Lp, Move::Up];
        
        let mut cube = Cube::new_solved();
        cube.apply_moves(&moves);

        cube.swap_corners(Corner::URB, Corner::URF);
        cube.swap_corners(Corner::URF, Corner::ULF);
        cube.corners[Corner::URF as usize].rotate(2);
        cube.corners[Corner::ULF as usize].rotate(2);
        cube.corners[Corner::URB as usize].rotate(2);

        assert!(cube.is_solved());
    }

    #[test]
    fn d_f_b_moves() {
        // Short algorithm that affects only few pieces in the end.
        let moves = [Move::Fp, Move::D, Move::B, Move::Dp, Move::F, Move::D, Move::Bp, Move::Dp];
        
        let mut cube = Cube::new_solved();
        cube.apply_moves(&moves);

        cube.swap_corners(Corner::DLF, Corner::DRF);
        cube.swap_corners(Corner::DRF, Corner::DRB);
        cube.corners[Corner::DRF as usize].rotate(2);
        cube.corners[Corner::DLF as usize].rotate(2);
        cube.corners[Corner::DRB as usize].rotate(2);

        assert!(cube.is_solved());
    }

    #[test]
    fn double_moves() {
        
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
}