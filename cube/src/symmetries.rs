use std::ops::Mul;

use super::Cube;

// ============================================================================
// Internal Representation (_Symmetry)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq)]
struct _Symmetry {
    cube_transform: Cube,
    is_reflection: bool,
}

// Symmetry Cube: Identity
const _IDENTITY: _Symmetry = _Symmetry {
    cube_transform: Cube::new_solved(),
    is_reflection: false,
};

// Symmetry Cube: 120-degree rotation around the URF-DLB axis (C3 rotation)
const ORDER_C3: usize = 3;
const _GENERATOR_C3: _Symmetry = _Symmetry {
    cube_transform: Cube {
        edge_perm: [4, 8, 5, 0, 3, 11, 9, 1, 7, 10, 6, 2],
        edge_orient: [1, 1, 1, 1, 0, 0, 0, 0, 1, 1, 1, 1],
        corner_perm: [0, 5, 4, 1, 6, 3, 2, 7],
        corner_orient: [1, 2, 1, 2, 1, 2, 1, 2],
    },
    is_reflection: false,
};

// Symmetry Cube: 180-degree rotation around F-B axis (C2 rotation)
const ORDER_C2: usize = 2;
const _GENERATOR_C2: _Symmetry = _Symmetry {
    cube_transform: Cube {
        edge_perm: [10, 9, 8, 11, 7, 6, 5, 4, 2, 1, 0, 3],
        edge_orient: [0; 12],
        corner_perm: [6, 7, 4, 5, 2, 3, 0, 1],
        corner_orient: [0; 8],
    },
    is_reflection: false,
};

// Symmetry Cube: 90-degree rotation around U-D axis (C4 rotation)
const ORDER_C4: usize = 4;
const _GENERATOR_C4: _Symmetry = _Symmetry {
    cube_transform: Cube {
        edge_perm: [3, 0, 1, 2, 7, 4, 5, 6, 11, 8, 9, 10],
        edge_orient: [0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0],
        corner_perm: [3, 0, 1, 2, 5, 6, 7, 4],
        corner_orient: [0; 8],
    },
    is_reflection: false,
};

// Symmetry Cube: Reflection across the U-D-F-B plane / M slice
const ORDER_SIGMA: usize = 2;
const _GENERATOR_SIGMA: _Symmetry = _Symmetry {
    cube_transform: Cube {
        edge_perm: [2, 1, 0, 3, 7, 6, 5, 4, 10, 9, 8, 11],
        edge_orient: [0; 12],
        corner_perm: [3, 2, 1, 0, 7, 6, 5, 4],
        corner_orient: [0; 8],
    },
    is_reflection: true,
};

impl _Symmetry {
    const fn multiply(sym1: &Self, sym2: &Self) -> Self {
        let mut new_transform = sym2.cube_transform;
        if sym1.is_reflection {
            let mut i = 0;
            while i < 8 {
                new_transform.corner_orient[i] = (3 - new_transform.corner_orient[i]) % 3;
                i += 1;
            }
        }
        new_transform = Cube::multiply(&sym1.cube_transform, &new_transform);

        let new_is_reflection = sym1.is_reflection ^ sym2.is_reflection;

        Self {
            cube_transform: new_transform,
            is_reflection: new_is_reflection,
        }
    }

    const fn eq(&self, other: &Self) -> bool {
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

// ============================================================================
// Precomputed Tables
// ============================================================================

// The above 4 generators have the special property that their cyclic subgroups
// intersect only at the identity, and every symmetry can be uniquely expressed
// as σ^l * c4^k * c2^j * c3^i where i,j,k,l range over each generator's order.
// This gives exactly 2 * 4 * 2 * 3 = 48 unique symmetries (semidirect product).
const SYMMETRY_MAP: [_Symmetry; NUM_SYMMETRIES] = {
    let mut result = [_IDENTITY; NUM_SYMMETRIES]; // ✓ Fixed

    let mut sym = _IDENTITY;
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
                    sym = _Symmetry::multiply(&_GENERATOR_SIGMA, &sym);
                    l += 1;
                }
                sym = _Symmetry::multiply(&_GENERATOR_C4, &sym);
                k += 1;
            }
            sym = _Symmetry::multiply(&_GENERATOR_C2, &sym);
            j += 1;
        }
        sym = _Symmetry::multiply(&_GENERATOR_C3, &sym);
        i += 1;
    }

    result
};

const MULTIPLICATION_TABLE: [[Symmetry; NUM_SYMMETRIES]; NUM_SYMMETRIES] = {
    let mut result = [[IDENTITY; NUM_SYMMETRIES]; NUM_SYMMETRIES];
    let mut i = 0;
    while i < NUM_SYMMETRIES {
        let sym1 = SYMMETRY_MAP[i];
        let mut j = 0;
        while j < NUM_SYMMETRIES {
            let sym2 = SYMMETRY_MAP[j];
            let product = _Symmetry::multiply(&sym1, &sym2);
            let mut k = 0;
            while k < NUM_SYMMETRIES {
                if product.eq(&SYMMETRY_MAP[k]) {
                    result[i][j] = Symmetry(k as u8);
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

const INV_INDEX_MAP: [Symmetry; NUM_SYMMETRIES] = {
    let mut result = [IDENTITY; NUM_SYMMETRIES];
    let mut i = 0;
    while i < NUM_SYMMETRIES {
        let mut j = 0;
        while j < NUM_SYMMETRIES {
            if MULTIPLICATION_TABLE[i][j].0 == IDENTITY.0 {
                result[i] = Symmetry(j as u8);
                break;
            }
            j += 1;
        }
        i += 1;
    }
    result
};

// ============================================================================
// Public API (Symmetry)
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Symmetry(u8);

pub const NUM_SYMMETRIES: usize = ORDER_C3 * ORDER_C2 * ORDER_C4 * ORDER_SIGMA;
pub const IDENTITY: Symmetry = Symmetry(0);
pub const GENERATOR_SIGMA: Symmetry = Symmetry(1);
pub const GENERATOR_C4: Symmetry = Symmetry(ORDER_SIGMA as u8);
pub const GENERATOR_C2: Symmetry = Symmetry((ORDER_C4 * ORDER_SIGMA) as u8);
pub const GENERATOR_C3: Symmetry = Symmetry((ORDER_C2 * ORDER_C4 * ORDER_SIGMA) as u8);

#[allow(non_upper_case_globals)]
pub const D4h_SYMMETRIES: [Symmetry; 16] = {
    let mut result = [IDENTITY; 16];
    let mut i: u8 = 0;
    while i < 16 {
        result[i as usize] = Symmetry(i);
        i += 1;
    }
    result
};

impl Symmetry {
    pub const fn from_u8_unchecked(val: u8) -> Self {
        Self(val)
    }

    pub const fn index(&self) -> usize {
        self.0 as usize
    }

    pub const fn inverse(&self) -> Self {
        INV_INDEX_MAP[self.index()]
    }

    pub fn cube_conjugation(cube: &Cube, sym: Symmetry) -> Cube {
        let sym_cube = _Symmetry {
            cube_transform: *cube,
            is_reflection: false,
        };

        let index = sym.index();
        let sym = SYMMETRY_MAP[index];
        let inv_sym = SYMMETRY_MAP[INV_INDEX_MAP[index].index()];
        // (sym * sym_cube * inv_sym).cube_transform
        _Symmetry::multiply(&_Symmetry::multiply(&sym, &sym_cube), &inv_sym).cube_transform
    }
}

impl Mul for Symmetry {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        MULTIPLICATION_TABLE[self.index()][rhs.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::super::Rng;
    use super::*;

    macro_rules! test_symmetry {
        ($symmetry_name:ident, $_sym:expr, $sym:expr, $order:expr) => {
            mod $symmetry_name {
                use super::*;

                #[test]
                fn test_generator_index() {
                    assert!(
                        SYMMETRY_MAP[$sym.index()] == $_sym,
                        "Generator index does not match expected symmetry"
                    );
                }

                #[test]
                fn test_orbit() {
                    let mut test = $sym;
                    for _ in 1..$order {
                        assert!(
                            test != IDENTITY,
                            "Should not return to identity before {} applications",
                            $order
                        );
                        test = $sym * test;
                    }
                    assert!(
                        test == IDENTITY,
                        "Should return to identity after {} applications",
                        $order
                    );
                }

                #[test]
                fn test_conjugation() {
                    let solved = Cube::new_solved();
                    let transformed = Symmetry::cube_conjugation(&solved, $sym);
                    assert!(
                        transformed.is_solved(),
                        "Conjugated solved cube should be solved"
                    );

                    let mut rng = Rng::with_seed(42);
                    for _ in 0..1000 {
                        let cube = Cube::new_random(&mut rng);
                        let transformed = Symmetry::cube_conjugation(&cube, $sym);
                        assert!(transformed.is_valid(), "Conjugated cube should be valid");
                    }
                }
            }
        };
    }

    test_symmetry!(c3, _GENERATOR_C3, GENERATOR_C3, ORDER_C3);
    test_symmetry!(c2, _GENERATOR_C2, GENERATOR_C2, ORDER_C2);
    test_symmetry!(c4, _GENERATOR_C4, GENERATOR_C4, ORDER_C4);
    test_symmetry!(sigma, _GENERATOR_SIGMA, GENERATOR_SIGMA, ORDER_SIGMA);

    #[test]
    fn test_uniqueness() {
        let mut seen = Vec::new();
        for (idx, &sym) in SYMMETRY_MAP.iter().enumerate() {
            for (prev_idx, prev_sym) in seen.iter() {
                assert!(
                    !sym.eq(prev_sym),
                    "_Symmetry {} is duplicate of symmetry {}",
                    idx,
                    prev_idx
                );
            }
            seen.push((idx, sym));
        }
    }
}
