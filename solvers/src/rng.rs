pub struct Rng {
    state: u64,
}

impl Rng {
    const DEFAULT_SEED: u64 = 42;
    
    pub fn new() -> Self {
        Self { state: Self::DEFAULT_SEED }
    }

    pub fn with_seed(seed: u64) -> Self {
        assert_ne!(seed, 0, "Seed cannot be zero for xorshift RNG");
        Self { state: seed }
    }

    #[inline]
    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    #[inline]
    pub fn u32(&mut self) -> u32 {
        self.next() as u32
    }

    #[inline]
    pub fn f32(&mut self) -> f32 {
        // Sample uniform in [0, 1)
        const SCALE: f32 = 1.0 / (u32::MAX as f32 + 1.0);
        (self.u32() as f32) * SCALE
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}

pub fn random_permutation<const N: usize>(rng: &mut Rng) -> [usize; N] {
    let mut arr = [0; N];
    for i in 0..N {
        arr[i] = i;
    }

    // Fisher-Yates shuffle results in uniform sampling
    for i in (1..N).rev() {
        // Uniformly sample number in [0, i]
        let j = (rng.u32() as usize) % (i + 1);
        // Technically the above j is not sampled truly uniformly, 
        // but since i << u32::MAX the bias is negligible.
        arr.swap(i, j);
    }
    arr
}