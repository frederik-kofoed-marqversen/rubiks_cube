use std::ops::Mul;

use crate::cube::Cube;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Symmetry {
    cube_transform: Cube,
    is_reflection: bool,
}

// Symmetry Cube: Identity
pub const IDENTITY: Symmetry = Symmetry {
    cube_transform: Cube::new_solved(),
    is_reflection: false,
};

// Symmetry Cube: 120-degree rotation around the URF-DLB axis (C3 rotation)
pub const ORDER_C3: usize = 3;
pub const GENERATOR_C3: Symmetry = Symmetry {
    cube_transform: Cube {
        edge_perm: [4, 8, 5, 0, 3, 11, 9, 1, 7, 10, 6, 2],
        edge_orient: [1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1],
        corner_perm: [0, 5, 4, 1, 6, 3, 2, 7],
        corner_orient: [1, 2, 1, 2, 1, 2, 1, 2],
    },
    is_reflection: false,
};

// Symmetry Cube: 180-degree rotation around F-B axis (C2 rotation)
pub const ORDER_C2: usize = 2;
pub const GENERATOR_C2: Symmetry = Symmetry {
    cube_transform: Cube {
        edge_perm: [10, 9, 8, 11, 7, 6, 5, 4, 2, 1, 0, 3],
        edge_orient: [0; 12],
        corner_perm: [6, 7, 4, 5, 2, 3, 0, 1],
        corner_orient: [0; 8],
    },
    is_reflection: false,
};

// Symmetry Cube: 90-degree rotation around U-D axis (C4 rotation)
pub const ORDER_C4: usize = 4;
pub const GENERATOR_C4: Symmetry = Symmetry {
    cube_transform: Cube {
        edge_perm: [3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10],
        edge_orient: [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0],
        corner_perm: [3, 0, 1, 2, 5, 6, 7, 4],
        corner_orient: [0; 8],
    },
    is_reflection: false,
};

// Symmetry Cube: Reflection across the U-D-F-B plane / M slice
pub const ORDER_SIGMA: usize = 2;
pub const GENERATOR_SIGMA: Symmetry = Symmetry {
    cube_transform: Cube {
        edge_perm: [2, 1, 0, 3, 7, 6, 5, 4, 10, 9, 8, 11],
        edge_orient: [0; 12],
        corner_perm: [3, 2, 1, 0, 7, 6, 5, 4],
        corner_orient: [0; 8],
    },
    is_reflection: true,
};

// These 4 generators have the special property that their cyclic subgroups
// intersect only at the identity, and every symmetry can be uniquely expressed
// as σ^l * c4^k * c2^j * c3^i where i,j,k,l range over each generator's order.
// This gives exactly 2 * 4 * 2 * 3 = 48 unique symmetries (semidirect product).
pub const GENERATORS: [Symmetry; 4] = [GENERATOR_C3, GENERATOR_C2, GENERATOR_C4, GENERATOR_SIGMA];
pub const NUM_SYMMETRIES: usize = ORDER_C3 * ORDER_C2 * ORDER_C4 * ORDER_SIGMA;

impl Cube {
    pub const fn apply_symmetry(&self, sym: &Symmetry) -> Cube {
        let mut result = *self;
        if sym.is_reflection {
            let mut i = 0;
            while i < 8 {
                result.corner_orient[i] = (3 - result.corner_orient[i]) % 3;
                i += 1;
            }
        }

        result = Cube::multiply(&sym.cube_transform, &result);
        result
    }
}

impl Mul<Cube> for Symmetry {
    type Output = Cube;

    fn mul(self, rhs: Cube) -> Self::Output {
        rhs.apply_symmetry(&self)
    }
}

impl Symmetry {
    pub const fn multiply(sym2: &Self, sym1: &Self) -> Self {
        let new_transform = sym1.cube_transform.apply_symmetry(&sym2);
        let new_is_reflection = sym1.is_reflection ^ sym2.is_reflection;
        Self {
            cube_transform: new_transform,
            is_reflection: new_is_reflection,
        }
    }

    pub const fn eq(&self, other: &Self) -> bool {
        if self.is_reflection != other.is_reflection {
            return false;
        }
        let mut i = 0;
        while i < 12 {
            if self.cube_transform.edge_perm[i] != other.cube_transform.edge_perm[i]
                || self.cube_transform.edge_orient[i] != other.cube_transform.edge_orient[i]
            {
                return false;
            }
            i += 1;
        }
        let mut i = 0;
        while i < 8 {
            if self.cube_transform.corner_perm[i] != other.cube_transform.corner_perm[i]
                || self.cube_transform.corner_orient[i] != other.cube_transform.corner_orient[i]
            {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl Mul for Symmetry {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::multiply(&self, &rhs)
    }
}

pub const SYMMETRIES: [Symmetry; NUM_SYMMETRIES] = {
    let mut result = [IDENTITY; NUM_SYMMETRIES]; // ✓ Fixed

    let mut sym = IDENTITY;
    let mut sym_idx = 0; // sym_idx = ((i*2 + j)*4 + k)*2 + l = i*16 + j*8 + k*2 + l
    let mut i = 0;
    while i < ORDER_C3 {
        let mut j = 0;
        while j < ORDER_C2 {
            let mut k = 0;
            while k < ORDER_C4 {
                let mut l = 0;
                while l < ORDER_SIGMA {
                    result[sym_idx] = sym;
                    sym_idx += 1;
                    sym = Symmetry::multiply(&GENERATOR_SIGMA, &sym);
                    l += 1;
                }
                sym = Symmetry::multiply(&GENERATOR_C4, &sym);
                k += 1;
            }
            sym = Symmetry::multiply(&GENERATOR_C2, &sym);
            j += 1;
        }
        sym = Symmetry::multiply(&GENERATOR_C3, &sym);
        i += 1;
    }

    result
};

pub const INV_INDEX_MAP: [usize; NUM_SYMMETRIES] = {
    let mut result = [usize::MAX; NUM_SYMMETRIES];
    let mut i = 0;
    while i < NUM_SYMMETRIES {
        let sym1 = SYMMETRIES[i];
        let mut j = 0;
        while j < NUM_SYMMETRIES {
            let sym2 = SYMMETRIES[j];
            let product = Symmetry::multiply(&sym1, &sym2);
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

pub const MULTIPLICATION_TABLE: [[usize; NUM_SYMMETRIES]; NUM_SYMMETRIES] = {
    let mut result = [[usize::MAX; NUM_SYMMETRIES]; NUM_SYMMETRIES];
    let mut i = 0;
    while i < NUM_SYMMETRIES {
        let sym1 = SYMMETRIES[i];
        let mut j = 0;
        while j < NUM_SYMMETRIES {
            let sym2 = SYMMETRIES[j];
            let product = Symmetry::multiply(&sym1, &sym2);
            let mut k = 0;
            while k < NUM_SYMMETRIES {
                if product.eq(&SYMMETRIES[k]) {
                    result[i][j] = k;
                    break;
                }
                k += 1;
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
                        assert!(
                            sym != IDENTITY,
                            "Should not return to identity before {} applications",
                            $order
                        );
                        sym = $generator * sym;
                    }
                    assert!(
                        sym == IDENTITY,
                        "Should return to identity after {} applications",
                        $order
                    );
                }
            }
        };
    }

    test_symmetry!(c3, GENERATOR_C3, ORDER_C3);
    test_symmetry!(c2, GENERATOR_C2, ORDER_C2);
    test_symmetry!(c4, GENERATOR_C4, ORDER_C4);
    test_symmetry!(sigma, GENERATOR_SIGMA, ORDER_SIGMA);

    #[test]
    fn test_inverses() {
        for idx in 0..NUM_SYMMETRIES {
            let sym = SYMMETRIES[idx];

            let inv_idx = INV_INDEX_MAP[idx];
            if inv_idx == usize::MAX {
                assert!(false, "Symmetry {idx} has no inverse!");
            }
            assert!(inv_idx != usize::MAX, "Symmetry {} has no inverse!", idx);

            let inv_sym = SYMMETRIES[inv_idx];
            assert!(
                sym * inv_sym == IDENTITY,
                "Symmetry {idx} should be inverted by {inv_idx}"
            );
        }
    }

    #[test]
    fn test_group_closure() {
        // Every product of two symmetries should be another symmetry
        for i in 0..NUM_SYMMETRIES {
            for j in 0..NUM_SYMMETRIES {
                let product = SYMMETRIES[i] * SYMMETRIES[j];
                let product_idx = MULTIPLICATION_TABLE[i][j];
                assert!(
                    product_idx != usize::MAX,
                    "Product of symmetries {i} and {j} is not in the symmetry table"
                );
                assert!(
                    product == SYMMETRIES[product_idx],
                    "Product of symmetries {i} and {j} should equal symmetry {product_idx}"
                );
            }
        }
    }
}
