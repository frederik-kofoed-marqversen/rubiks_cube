use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// A 1D lookup table backed by a Vec
#[derive(Serialize, Deserialize, Clone)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct LookupTable<T> {
    pub data: Vec<T>,
}

impl<T> LookupTable<T> {
    pub fn new(size: usize, default: T) -> Self
    where
        T: Clone,
    {
        Self {
            data: vec![default; size],
        }
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }
}

/// A 2D lookup table specialization that stores data in row-major order
#[derive(Serialize, Deserialize, Clone)]
#[serde(bound(serialize = "T: Serialize", deserialize = "T: Deserialize<'de>"))]
pub struct LookupTable2D<T, const IDX2_WIDTH: usize> {
    table: LookupTable<T>,
}

impl<T, const IDX2_WIDTH: usize> LookupTable2D<T, IDX2_WIDTH> {
    pub fn new(idx1_width: usize, default: T) -> Self
    where
        T: Clone,
    {
        Self {
            table: LookupTable::new(idx1_width * IDX2_WIDTH, default),
        }
    }

    pub fn idx1_width(&self) -> usize {
        self.table.len() / IDX2_WIDTH
    }

    pub fn idx2_width(&self) -> usize {
        IDX2_WIDTH
    }
}

// Deref to the inner LookupTable so that .data field is accessible
impl<T, const IDX2_WIDTH: usize> Deref for LookupTable2D<T, IDX2_WIDTH> {
    type Target = LookupTable<T>;

    fn deref(&self) -> &Self::Target {
        &self.table
    }
}

impl<T, const IDX2_WIDTH: usize> DerefMut for LookupTable2D<T, IDX2_WIDTH> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.table
    }
}
