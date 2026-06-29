const BINOM: [[usize; 5]; 13] = precompute_binomials();
const FACTORIAL: [usize; 13] = precompute_factorials();
const MOD3_DISTANCE_MAP: [[u32; 3]; 21] = precompute_mod3_distance_map();

// Precompute binomial coefficients using Pascal's triangle.
const fn precompute_binomials<const N: usize, const K: usize>() -> [[usize; K]; N] {
    let mut table = [[0; K]; N];

    let mut n = 0;
    while n < N {
        let mut k = 0;
        while k < K {
            if k == 0 || k == n {
                table[n][k] = 1;
            } else if k < n {
                table[n][k] = table[n - 1][k - 1] + table[n - 1][k];
            } else {
                table[n][k] = 0;
            }
            k += 1;
        }
        n += 1;
    }

    table
}

const fn precompute_factorials<const N: usize>() -> [usize; N] {
    let mut table = [1; N];
    let mut i = 1;
    while i < N {
        table[i] = table[i - 1] * i;
        i += 1;
    }
    table
}

const fn precompute_mod3_distance_map<const MAX_DIST: usize>() -> [[u32; 3]; MAX_DIST] {
    let mut map = [[0; 3]; MAX_DIST];
    
    // Handle i = 0 case (never accessed in practice - sentinel value)
    map[0][0] = 0;
    map[0][1] = 1;
    map[0][2] = u32::MAX; // invalid transition implies new distance is -1
    
    let mut i = 1;
    while i < MAX_DIST {
        map[i][(i - 1) % 3] = (i - 1) as u32;
        map[i][i % 3] = i as u32;
        map[i][(i + 1) % 3] = (i + 1) as u32;
        i += 1;
    }
    map
}

pub fn compute_permutation<T: PartialEq + Copy, const N: usize>(
    initial: &[T; N],
    permutation: &[T; N],
) -> [usize; N] {
    let mut result = [0; N];

    for i in 0..N {
        result[i] = initial
            .iter()
            .position(|&x| x == permutation[i])
            .expect("Not a valid permutation");
    }

    result
}

pub fn permutation_parity<const N: usize>(permutation: &[usize; N]) -> usize {
    let mut visited = [false; N];
    let mut parity = 0;

    for mut i in 0..N {
        if visited[i] {
            continue;
        }

        while !visited[i] {
            visited[i] = true;
            i = permutation[i];
            parity ^= 1;
        }
        parity ^= 1;
    }

    parity
}

pub const fn permutation_inverse<const N: usize>(permutation: &[usize; N]) -> [usize; N] {
    let mut inverse = [0; N];
    let mut i = 0;
    while i < N {
        inverse[permutation[i]] = i;
        i += 1;
    }
    inverse
}

/// Encodes a permutation of N distinct elements into its lexicographic rank.
/// The rank is computed using the factorial number system, where each position's
/// contribution is determined by how many smaller available elements are to the right of it.
pub fn permutation_rank<const N: usize>(permutation: &[usize; N]) -> usize {
    let mut available = [true; N];
    let mut rank = 0;

    for i in 0..N {
        let elem = permutation[i];
        // Count how many available elements are smaller than elem
        let digit = (0..elem).filter(|&x| available[x]).count();
        rank += digit * FACTORIAL[N - 1 - i];
        available[elem] = false;
    }
    rank
}

pub fn permutation_unrank<const N: usize>(mut rank: usize) -> [usize; N] {
    let mut available = [true; N];
    let mut result = [0; N];

    for i in 0..N {
        let fact = FACTORIAL[N - 1 - i];
        let digit = rank / fact;
        rank %= fact;

        // Find the digit-th available element
        let elem = (0..N).filter(|&j| available[j]).nth(digit).unwrap();
        result[i] = elem;
        available[elem] = false;
    }
    result
}

/// Encodes a combination (subset selection) into its combinatorial rank.
///
/// Given N positions and K selected positions (marked true in the array),
/// computes the lexicographic rank using the combinatorial number system.
///
/// Formula: rank = Σ C(i, k) for each selected position i (k is 1-indexed count)
pub fn combination_rank<const N: usize, const K: usize>(selected: &[bool; N]) -> usize {
    let mut rank = 0;
    let mut k = 0; // Number of selected elements found so far

    for i in 0..N {
        if k == K {
            break;
        }
        if selected[i] {
            k += 1;
            rank += BINOM[i][k]; // C(i, k)
        }
    }
    rank
}

pub fn combination_unrank<const N: usize, const K: usize>(mut rank: usize) -> [bool; N] {
    let mut selected = [false; N];
    let mut remaining = K; // Number of elements left to select

    for i in (0..N).rev() {
        if remaining == 0 {
            break;
        }
        let b = BINOM[i][remaining];
        if rank >= b {
            // Position i is selected
            selected[i] = true;
            rank -= b;
            remaining -= 1;
        }
    }
    selected
}

pub fn update_distance_mod3(current_distance: u32, next_distance_mod3: u32) -> u32 {
    MOD3_DISTANCE_MAP[current_distance as usize][next_distance_mod3 as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precompute_binomials() {
        const BINOM: [[usize; 5]; 13] = precompute_binomials();
        assert_eq!(BINOM[5][2], 10);
        assert_eq!(BINOM[6][3], 20);
        assert_eq!(BINOM[10][4], 210);
        assert_eq!(BINOM[11][4], 330);
        assert_eq!(BINOM[12][4], 495);
    }

    #[test]
    fn test_precompute_factorials() {
        const FACT: [usize; 6] = precompute_factorials();
        assert_eq!(FACT[0], 1);
        assert_eq!(FACT[1], 1);
        assert_eq!(FACT[2], 2);
        assert_eq!(FACT[3], 6);
        assert_eq!(FACT[4], 24);
        assert_eq!(FACT[5], 120);
    }

    #[test]
    fn test_permutation_rank_unrank() {
        for i in 0..100 {
            let perm = permutation_unrank::<5>(i);
            let rank = permutation_rank::<5>(&perm);
            assert_eq!(i, rank, "Permutation rank/unrank mismatch for rank {}", i);
        }
    }

    #[test]
    fn test_combination_rank_unrank() {
        for i in 0..100 {
            let comb = combination_unrank::<10, 3>(i);
            let rank = combination_rank::<10, 3>(&comb);
            assert_eq!(i, rank, "Combination rank/unrank mismatch for rank {}", i);
        }
    }

    #[test]
    fn test_update_distance_mod3() {
        assert_eq!(update_distance_mod3(0, 0), 0);
        assert_eq!(update_distance_mod3(0, 1), 1);
        
        for current in 1..10 {
            for next in current-1..=current+1 {
                let next_mod3 = next % 3;
                let updated = update_distance_mod3(current, next_mod3);
                assert_eq!(updated, next, "Distance update mismatch for current {} and next {}", current, next);
            }
        }
    }
}
