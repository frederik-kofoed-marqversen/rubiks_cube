use std::ops::Mul;

use crate::cube::Cube;
use crate::math::compose_permutations;

// Symmetry Cube: Identity
pub const IDENTITY: Cube = Cube::new_solved();

// Symmetry Cube: 90-degree rotation around U-D axis (C4 rotation)
pub const C4_GENERATOR: Cube = Cube {
    edge_perm: [3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10],
    edge_orient: [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0],
    corner_perm: [3, 0, 1, 2, 7, 4, 5, 6],
    corner_orient: [0; 8],
};

// Symmetry Cube: 180-degree rotation around F-B axis (C2 rotation)
pub const C2_GENERATOR: Cube = Cube {
    edge_perm: [10, 9, 8, 11, 7, 6, 5, 4, 2, 1, 0, 3],
    edge_orient: [0; 12],
    corner_perm: [6, 7, 4, 5, 2, 3, 0, 1],
    corner_orient: [0; 8],
};

// Symmetry Cube: Reflection across the U-D-F-B plane / M slice
pub const SIGMA_GENERATOR: Cube = Cube {
    edge_perm: [2, 3, 0, 1, 7, 6, 5, 4, 10, 11, 8, 9],
    edge_orient: [0; 12],
    corner_perm: [3, 2, 1, 0, 7, 6, 5, 4],
    corner_orient: [0; 8],
};

// Symmetry Cube: 120-degree rotation around the URF-DLB axis (C3 rotation)
pub const C3_GENERATOR: Cube = Cube {
    edge_perm: [4, 8, 5, 0, 3, 11, 9, 1, 7, 10, 6, 2],
    edge_orient: [1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1],
    corner_perm: [0, 5, 4, 1, 6, 3, 2, 7],
    corner_orient: [1, 2, 1, 2, 1, 2, 1, 2],
};

// These 4 generators have the special property that their cyclic subgroups
// intersect only at the identity, and every symmetry can be uniquely expressed
// as c3^i · c2^j · c4^k · σ^l where i,j,k,l range over each generator's order.
// This gives exactly 3 × 2 × 4 × 2 = 48 unique symmetries (semidirect product).
pub const GENERATORS: [Cube; 4] = [C3_GENERATOR, C2_GENERATOR, C4_GENERATOR, SIGMA_GENERATOR];

impl Cube {
    /// Multiply (compose) two cubes
    /// Using standard composition notation: multiply(cube2, cube1) = (cube2 ∘ cube1)(x) = cube2(cube1(x))
    pub const fn multiply(cube2: &Cube, cube1: &Cube) -> Cube {
        let edge_perm = compose_permutations(&cube2.edge_perm, &cube1.edge_perm);
        let corner_perm = compose_permutations(&cube2.corner_perm, &cube1.corner_perm);

        let mut edge_orient = [0u32; 12];
        let mut i = 0;
        while i < 12 {
            edge_orient[i] = (cube1.edge_orient[i] + cube2.edge_orient[cube1.edge_perm[i]]) % 2;
            i += 1;
        }

        let mut corner_orient = [0u32; 8];
        let mut i = 0;
        while i < 8 {
            corner_orient[i] =
                (cube1.corner_orient[i] + cube2.corner_orient[cube1.corner_perm[i]]) % 3;
            i += 1;
        }

        Cube {
            edge_perm,
            edge_orient,
            corner_perm,
            corner_orient,
        }
    }

    pub const fn eq(&self, other: &Self) -> bool {
        // Check edges
        let mut i = 0;
        while i < 12 {
            if self.edge_perm[i] != other.edge_perm[i]
                || self.edge_orient[i] != other.edge_orient[i]
            {
                return false;
            }
            i += 1;
        }
        // Check corners
        let mut i = 0;
        while i < 8 {
            if self.corner_perm[i] != other.corner_perm[i]
                || self.corner_orient[i] != other.corner_orient[i]
            {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl Mul for Cube {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Cube::multiply(&rhs, &self)
    }
}

pub const SYMMETRIES: [Cube; 48] = {
    let mut result = [IDENTITY; 48]; // ✓ Fixed

    let mut sym = IDENTITY;
    let mut sym_idx = 0;
    let mut i = 0;
    while i < 3 {
        let mut j = 0;
        while j < 2 {
            let mut k = 0;
            while k < 4 {
                let mut l = 0;
                while l < 2 {
                    result[sym_idx] = sym;
                    sym_idx += 1;
                    sym = Cube::multiply(&SIGMA_GENERATOR, &sym);
                    l += 1;
                }
                sym = Cube::multiply(&C4_GENERATOR, &sym);
                k += 1;
            }
            sym = Cube::multiply(&C2_GENERATOR, &sym);
            j += 1;
        }
        sym = Cube::multiply(&C3_GENERATOR, &sym);
        i += 1;
    }

    result
};

pub const INV_MAP: [usize; 48] = {
    let mut result = [usize::MAX; 48];
    let mut i = 0;
    while i < 48 {
        let sym1 = SYMMETRIES[i];
        let mut j = 0;
        while j < 48 {
            let sym2 = SYMMETRIES[j];
            let product = Cube::multiply(&sym1, &sym2);
            if product.eq(&IDENTITY) {
                result[i] = j;
                break;
            }
            j += 1;
        }
        i += 1;
    }
    result
};

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_symmetry {
        ($symmetry_name:ident, $generator:expr, $order:expr) => {
            mod $symmetry_name {
                use super::*;

                #[test]
                fn test_symmetry() {
                    let mut sym = $generator;
                    for _ in 1..$order {
                        assert_ne!(
                            sym, IDENTITY,
                            "Should not return to identity before {} applications",
                            $order
                        );
                        sym = $generator * sym;
                    }
                    assert_eq!(
                        sym, IDENTITY,
                        "Should return to identity after {} applications",
                        $order
                    );
                }
            }
        };
    }

    test_symmetry!(c4, C4_GENERATOR, 4);
    test_symmetry!(c2, C2_GENERATOR, 2);
    test_symmetry!(sigma, SIGMA_GENERATOR, 2);
    test_symmetry!(c3, C3_GENERATOR, 3);

    #[test]
    fn test_inverses() {
        for idx in 0..48 {
            let sym = SYMMETRIES[idx];

            let inv_idx = INV_MAP[idx];
            assert!(inv_idx != usize::MAX, "Symmetry {} has no inverse!", idx);

            let inv_sym = SYMMETRIES[inv_idx];
            assert_eq!(
                sym * inv_sym,
                IDENTITY,
                "Symmetry {} should be inverted by {}",
                idx,
                inv_idx
            );
        }
    }
}
