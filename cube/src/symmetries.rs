use crate::cube::Cube;

// Symmetry Cube: Identity
pub const SYM_I: Cube = Cube::new_solved();

// Symmetry Cube: 90-degree rotation around U-D axis (C4 rotation)
pub const SYM_C4: Cube = {
    const EDGE_PERM: [usize; 12] = [3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10];
    const EDGE_ORIENT: [u32; 12] = [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0];
    const CORNER_PERM: [usize; 8] = [3, 0, 1, 2, 7, 4, 5, 6];
    const CORNER_ORIENT: [u32; 8] = [0; 8];

    Cube::from_arrays(EDGE_PERM, EDGE_ORIENT, CORNER_PERM, CORNER_ORIENT)
};

// Symmetry Cube: 180-degree rotation around F-B axis (C2 rotation)
pub const SYM_C2: Cube = {
    const EDGE_PERM: [usize; 12] = [10, 9, 8, 11, 7, 6, 5, 4, 2, 1, 0, 3];
    const EDGE_ORIENT: [u32; 12] = [0; 12];
    const CORNER_PERM: [usize; 8] = [6, 7, 4, 5, 2, 3, 0, 1];
    const CORNER_ORIENT: [u32; 8] = [0; 8];

    Cube::from_arrays(EDGE_PERM, EDGE_ORIENT, CORNER_PERM, CORNER_ORIENT)
};

// Symmetry Cube: Reflection across the U-D-F-B plane / M slice
pub const SYM_SIGMA: Cube = {
    const EDGE_PERM: [usize; 12] = [2, 3, 0, 1, 7, 6, 5, 4, 10, 11, 8, 9];
    const EDGE_ORIENT: [u32; 12] = [0; 12];
    const CORNER_PERM: [usize; 8] = [3, 2, 1, 0, 7, 6, 5, 4];
    const CORNER_ORIENT: [u32; 8] = [0; 8];

    Cube::from_arrays(EDGE_PERM, EDGE_ORIENT, CORNER_PERM, CORNER_ORIENT)
};

// Symmetry Cube: 120-degree rotation around the URF-DLB axis (C3 rotation)
pub const SYM_C3: Cube = {
    const EDGE_PERM: [usize; 12] = [4, 8, 5, 0, 3, 11, 9, 1, 7, 10, 6, 2];
    const EDGE_ORIENT: [u32; 12] = [1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1];
    const CORNER_PERM: [usize; 8] = [0, 5, 4, 1, 6, 3, 2, 7];
    const CORNER_ORIENT: [u32; 8] = [1, 2, 1, 2, 1, 2, 1, 2];

    Cube::from_arrays(EDGE_PERM, EDGE_ORIENT, CORNER_PERM, CORNER_ORIENT)
};

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_symmetry {
        ($mod_name:ident, $symmetry:expr, $orbit:expr) => {
            mod $mod_name {
                use super::*;

                #[test]
                fn orbit() {
                    let mut sym = SYM_I;
                    for _ in 0..$orbit {
                        sym = Cube::multiply(&$symmetry, &sym);
                    }
                    assert_eq!(
                        sym, SYM_I,
                        "Should return to identity after {} applications",
                        $orbit
                    );
                }
            }
        };
    }

    test_symmetry!(c4, SYM_C4, 4);
    test_symmetry!(c2, SYM_C2, 2);
    test_symmetry!(sigma, SYM_SIGMA, 2);
    test_symmetry!(c3, SYM_C3, 3);
}
