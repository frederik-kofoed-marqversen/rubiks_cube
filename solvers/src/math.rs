pub const fn binom_const(n: usize, k: usize) -> usize {
    if k > n {
        return 0;
    }
    
    let mut result = 1;
    let mut i = 0;
    while i < k {
        result *= (n - i) / (i + 1);
        i += 1;
    }

    result
}

pub const fn precompute_binomials<const N: usize, const K: usize>() -> [[usize; K]; N] {
    let mut table = [[0; K]; N];
    let mut n = 0;
    while n < N {
        let mut k = 0;
        while k < K {
            table[n][k] = binom_const(n, k);
            k += 1;
        }
        n += 1;
    }
    table
}
