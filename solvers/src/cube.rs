#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    UR, UB, UL, UF, // top layer
    RF, RB, LB, LF, // middle layer
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

#[derive(Debug, Clone, Copy)]
pub enum Move {
    U, Up, U2,
    L, Lp, L2,
    D, Dp, D2,
    R, Rp, R2,
    F, Fp, F2,
    B, Bp, B2,
}

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
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Cubie<T> {
    piece_type: T,
    orientation: u8,
}

impl Cubie<Edge> {
    fn flip(&mut self) {
        // Addition mod 2
        self.orientation ^= 1;
    }
}

impl Cubie<Corner> {
    fn rotate(&mut self, amount: u8) {
        // Addition mod 3
        const MOD3: [u8; 5] = [0, 1, 2, 0, 1];
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

impl Cube {
    pub fn new() -> Self {
        Cube{
            edges: EDGES.map(|edge| Cubie{piece_type: edge, orientation: 0}),
            corners: CORNERS.map(|corner| Cubie{piece_type: corner, orientation: 0}),
            edge_pos: EDGES,
            corner_pos: CORNERS,
        }
    }

    pub fn is_solved(&self) -> bool {
        for (i, edge) in self.edges.iter().enumerate() {
            if edge.piece_type as usize != i || edge.orientation != 0 {
                return false
            }
        }
        for (i, corner) in self.corners.iter().enumerate() {
            if corner.piece_type as usize != i || corner.orientation != 0 {
                return false
            }
        }
        return true
    }

    pub fn get_edge_orientation(&self, pos: Edge) -> u8 {
        self.edges[pos as usize].orientation
    }

    pub fn get_edge_type(&self, pos: Edge) -> Edge {
        self.edges[pos as usize].piece_type
    }

    pub fn get_edge_position(&self, edge: Edge) -> Edge {
        self.edge_pos[edge as usize]
    }

    pub fn get_corner_orientation(&self, pos: Corner) -> u8 {
        self.corners[pos as usize].orientation
    }

    pub fn get_corner_type(&self, pos: Corner) -> Corner {
        self.corners[pos as usize].piece_type
    }

    pub fn get_corner_position(&self, corner: Corner) -> Corner {
        self.corner_pos[corner as usize]
    }

    pub fn turn(&mut self, turn: Move) -> &mut Self {
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

    pub fn apply_moves(&mut self, moves: &[Move]) -> &mut Self {
        for &turn in moves {
            self.turn(turn);
        }
        self
    }

    fn swap_edges(&mut self, pos1: Edge, pos2: Edge) {
        self.edges.swap(pos1 as usize, pos2 as usize);
        self.edge_pos.swap(self.edges[pos1 as usize].piece_type as usize, self.edges[pos2 as usize].piece_type as usize);
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r_l_u_moves() {
        // Short algorithm that affects only few pieces in the end.
        let moves = [Move::Rp, Move::U, Move::L, Move::Up, Move::R, Move::U, Move::Lp, Move::Up];
        
        let mut cube = Cube::new();
        for &turn in &moves {
            cube.turn(turn);
        }

        cube.corners.swap(Corner::URB as usize, Corner::URF as usize);
        cube.corners.swap(Corner::URF as usize, Corner::ULF as usize);
        cube.corners[Corner::URF as usize].rotate(2);
        cube.corners[Corner::ULF as usize].rotate(2);
        cube.corners[Corner::URB as usize].rotate(2);

        assert_eq!(cube, Cube::new());
    }

    #[test]
    fn d_f_b_moves() {
        // Short algorithm that affects only few pieces in the end.
        let moves = [Move::Fp, Move::D, Move::B, Move::Dp, Move::F, Move::D, Move::Bp, Move::Dp];
        
        let mut cube = Cube::new();
        for &turn in &moves {
            cube.turn(turn);
        }
        dbg!(&cube.edges);

        cube.corners.swap(Corner::DLF as usize, Corner::DRF as usize);
        cube.corners.swap(Corner::DRF as usize, Corner::DRB as usize);
        cube.corners[Corner::DRF as usize].rotate(2);
        cube.corners[Corner::DLF as usize].rotate(2);
        cube.corners[Corner::DRB as usize].rotate(2);

        assert_eq!(cube, Cube::new());
    }

    #[test]
    fn double_moves() {
        
        let mut cube1 = Cube::new();
        let mut cube2 = Cube::new();
        
        let moves = vec![Move::R2, Move::U2].repeat(3);
        for turn in moves {
            cube1.turn(turn);
        }

        cube2.edges.swap(Edge::UF as usize, Edge::UB as usize);
        cube2.edges.swap(Edge::RF as usize, Edge::RB as usize);

        assert_eq!(cube1, cube2);

        let moves = vec![Move::L2, Move::F2].repeat(3);
        for turn in moves {
            cube1.turn(turn);
        }

        cube2.edges.swap(Edge::UF as usize, Edge::DF as usize);
        cube2.edges.swap(Edge::UL as usize, Edge::DL as usize);

        assert_eq!(cube1, cube2);

        let moves = vec![Move::D2, Move::B2].repeat(3);
        for turn in moves {
            cube1.turn(turn);
        }

        cube2.edges.swap(Edge::RB as usize, Edge::LB as usize);
        cube2.edges.swap(Edge::DR as usize, Edge::DL as usize);

        assert_eq!(cube1, cube2);
    }
}