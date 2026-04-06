// Precompute binomial coefficients using Pascal's triangle.
pub const fn precompute_binomials<const N: usize, const K: usize>() -> [[usize; K]; N] {
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
}
