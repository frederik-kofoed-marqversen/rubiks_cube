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

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct Cubie<T> {
    piece_type: T,
    orientation: u8,
}

impl Cubie<Edge> {
    fn flip(&mut self) {
        if self.orientation == 1 {
            self.orientation = 0;
        } else {
            self.orientation = 1;
        }
    }
}

impl Cubie<Corner> {
    fn rotate(&mut self, amount: u8) {
        // When amount is 0, 1, or 2, this is addition mod 3
        self.orientation += amount;
        if self.orientation == 3 {
            self.orientation = 0;
        } else if self.orientation == 4 {
            self.orientation = 1;
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Cube {
    edges: [Cubie<Edge>; 12],
    corners: [Cubie<Corner>; 8],
}

impl Cube {
    pub fn new() -> Self {
        Cube{
            edges: EDGES.map(|edge| Cubie{piece_type: edge, orientation: 0}),
            corners: CORNERS.map(|corner| Cubie{piece_type: corner, orientation: 0})
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

    pub fn get_edge_orientation(&self, pos: &Edge) -> u8 {
        self.edges[*pos as usize].orientation
    }

    pub fn get_edge_type(&self, pos: &Edge) -> &Edge {
        &self.edges[*pos as usize].piece_type
    }

    pub fn get_edge_position(&self, edge: &Edge) -> &Edge {
        EDGES.iter().find(|&&pos| self.edges[pos as usize].piece_type == *edge).unwrap()
    }

    pub fn get_corner_orientation(&self, pos: &Corner) -> u8 {
        self.corners[*pos as usize].orientation
    }

    pub fn get_corner_type(&self, pos: &Corner) -> &Corner {
        &self.corners[*pos as usize].piece_type
    }

    pub fn get_corner_position(&self, corner: &Corner) -> &Corner {
        CORNERS.iter().find(|&&pos| self.corners[pos as usize].piece_type == *corner).unwrap()
    }

    pub fn turn(&mut self, turn: &Move) -> &mut Self {
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

    fn u(&mut self) -> &mut Self {
        self.edges.swap(Edge::UR as usize, Edge::UB as usize);
        self.edges.swap(Edge::UB as usize, Edge::UL as usize);
        self.edges.swap(Edge::UL as usize, Edge::UF as usize);
        
        self.corners.swap(Corner::URF as usize, Corner::URB  as usize);
        self.corners.swap(Corner::URB as usize, Corner::ULB  as usize);
        self.corners.swap(Corner::ULB as usize, Corner::ULF  as usize);
        
        self
    }

    fn d(&mut self) -> &mut Self {
        self.edges.swap(Edge::DR as usize, Edge::DF as usize);
        self.edges.swap(Edge::DF as usize, Edge::DL as usize);
        self.edges.swap(Edge::DL as usize, Edge::DB as usize);
        
        self.corners.swap(Corner::DRF as usize, Corner::DLF  as usize);
        self.corners.swap(Corner::DLF as usize, Corner::DLB  as usize);
        self.corners.swap(Corner::DLB as usize, Corner::DRB  as usize);
        
        self
    }

    fn r(&mut self) -> &mut Self {
        self.edges.swap(Edge::UR as usize, Edge::RF as usize);
        self.edges.swap(Edge::RF as usize, Edge::DR as usize);
        self.edges.swap(Edge::DR as usize, Edge::RB as usize);

        self.corners.swap(Corner::URF as usize, Corner::DRF as usize);
        self.corners.swap(Corner::DRF as usize, Corner::DRB as usize);
        self.corners.swap(Corner::DRB as usize, Corner::URB as usize);

        self.corners[Corner::URF as usize].rotate(1);
        self.corners[Corner::DRF as usize].rotate(2);
        self.corners[Corner::DRB as usize].rotate(1);
        self.corners[Corner::URB as usize].rotate(2);

        self
    }

    fn l(&mut self) -> &mut Self {
        self.edges.swap(Edge::UL as usize, Edge::LB as usize);
        self.edges.swap(Edge::LB as usize, Edge::DL as usize);
        self.edges.swap(Edge::DL as usize, Edge::LF as usize);

        self.corners.swap(Corner::ULF as usize, Corner::ULB as usize);
        self.corners.swap(Corner::ULB as usize, Corner::DLB as usize);
        self.corners.swap(Corner::DLB as usize, Corner::DLF as usize);

        self.corners[Corner::ULF as usize].rotate(2);
        self.corners[Corner::DLF as usize].rotate(1);
        self.corners[Corner::DLB as usize].rotate(2);
        self.corners[Corner::ULB as usize].rotate(1);

        self
    }

    fn f(&mut self) -> &mut Self {
        self.edges.swap(Edge::UF as usize, Edge::LF as usize);
        self.edges.swap(Edge::LF as usize, Edge::DF as usize);
        self.edges.swap(Edge::DF as usize, Edge::RF as usize);

        self.corners.swap(Corner::URF as usize, Corner::ULF as usize);
        self.corners.swap(Corner::ULF as usize, Corner::DLF as usize);
        self.corners.swap(Corner::DLF as usize, Corner::DRF as usize);

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
        self.edges.swap(Edge::UB as usize, Edge::RB as usize);
        self.edges.swap(Edge::RB as usize, Edge::DB as usize);
        self.edges.swap(Edge::DB as usize, Edge::LB as usize);

        self.corners.swap(Corner::URB as usize, Corner::DRB as usize);
        self.corners.swap(Corner::DRB as usize, Corner::DLB as usize);
        self.corners.swap(Corner::DLB as usize, Corner::ULB as usize);

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
        for turn in &moves {
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
        for turn in &moves {
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
            cube1.turn(&turn);
        }

        cube2.edges.swap(Edge::UF as usize, Edge::UB as usize);
        cube2.edges.swap(Edge::RF as usize, Edge::RB as usize);

        assert_eq!(cube1, cube2);

        let moves = vec![Move::L2, Move::F2].repeat(3);
        for turn in moves {
            cube1.turn(&turn);
        }

        cube2.edges.swap(Edge::UF as usize, Edge::DF as usize);
        cube2.edges.swap(Edge::UL as usize, Edge::DL as usize);

        assert_eq!(cube1, cube2);

        let moves = vec![Move::D2, Move::B2].repeat(3);
        for turn in moves {
            cube1.turn(&turn);
        }

        cube2.edges.swap(Edge::RB as usize, Edge::LB as usize);
        cube2.edges.swap(Edge::DR as usize, Edge::DL as usize);

        assert_eq!(cube1, cube2);
    }
}