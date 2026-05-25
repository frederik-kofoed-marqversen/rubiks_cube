pub struct Rng {
    state: u64,
}

impl Rng {
    const DEFAULT_SEED: u64 = 42;

    pub fn new() -> Self {
        Self {
            state: Self::DEFAULT_SEED,
        }
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
        // Sample uniform float in [0, 1)
        const SCALE: f32 = 1.0 / (u32::MAX as f32 + 1.0);
        (self.u32() as f32) * SCALE
    }

    #[inline]
    pub fn u32_range(&mut self, high: u32) -> u32 {
        // Simple modulo method - may have bias if high does not divide 2^32
        if high == 0 {
            return 0;
        }
        self.u32() % high
    }

    #[inline]
    pub fn u32_range_unbiased(&mut self, high: u32) -> u32 {
        // Swift's optimal algorithm: https://github.com/swiftlang/swift/pull/39143
        // Computes floor(random * high / 2^32) with negligible bias

        if high == 0 {
            return 0;
        }

        let random = self.u32() as u64;
        let mut m = random * (high as u64);
        let mut low: u32 = m as u32; // low 32 bits

        if low < high {
            // Need refinement - add 32 more bits of precision
            let random2 = self.u32() as u64;
            let m2 = random2 * (high as u64);

            // Add the low bits, detecting carry
            let old_low = low;
            low = low.wrapping_add(m2 as u32);

            // Add high bits and carry
            m += m2 >> 32;
            if low < old_low {
                // carry occurred
                m += 1;
            }
        }

        (m >> 32) as u32
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
        let j = rng.u32_range(i as u32 + 1) as usize;
        arr.swap(i, j);
    }
    arr
}
