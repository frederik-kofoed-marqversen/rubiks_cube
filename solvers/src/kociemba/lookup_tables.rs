struct LookupTable2D<T: Copy, const IDX2_WIDTH: usize> {
    data: Vec<T>,
}

impl<T: Copy, const IDX2_WIDTH: usize> LookupTable2D<T, IDX2_WIDTH> {
    #[inline]
    pub fn new(idx1_width: usize, default: T) -> Self {
        Self {
            data: vec![default; idx1_width * IDX2_WIDTH],
        }
    }

    #[inline(always)]
    pub fn get(&self, idx1: usize, idx2: usize) -> T {
        self.data[idx1 * IDX2_WIDTH + idx2]
    }

    #[inline(always)]
    pub fn set(&mut self, idx1: usize, idx2: usize, val: T) {
        self.data[idx1 * IDX2_WIDTH + idx2] = val;
    }
}

type MoveTable = LookupTable2D<u16, 18>;
type PruningTable<const IDX2_WIDTH: usize> = LookupTable2D<u8, IDX2_WIDTH>;