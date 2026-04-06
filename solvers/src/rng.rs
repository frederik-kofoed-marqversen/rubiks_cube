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
    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    #[inline]
    pub fn u32(&mut self) -> u32 {
        self.next_u64() as u32
    }

    #[inline]
    pub fn f32(&mut self) -> f32 {
        // uniform in [0, 1)
        const SCALE: f32 = 1.0 / (u32::MAX as f32 + 1.0);
        (self.u32() as f32) * SCALE
    }
}

impl Default for Rng {
    fn default() -> Self {
        Self::new()
    }
}