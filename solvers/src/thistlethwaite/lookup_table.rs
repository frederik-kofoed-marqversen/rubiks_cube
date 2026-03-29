use super::stages::Stage;
use super::cube::Cube;
use std::io::{Read, Write};
use std::marker::PhantomData;
use std::collections::VecDeque;

pub struct LookupTable<S> {
    data: Box<[u8]>,
    stage: PhantomData<S>
}

impl<'a, S: Stage<'a>> LookupTable<S> {
    /// Build a lookup table from scratch using BFS (fast, recommended)
    pub fn build() -> Self {
        println!("Building lookup table for stage (size: {})...", S::SIZE);
        Self {
            data: Self::build_table_bfs(),
            stage: PhantomData
        }
    }

    /// Load a precomputed lookup table from a file
    pub fn load(file_path: &str) -> Result<Self, std::io::Error> {
        let mut file = std::fs::File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if buffer.len() != S::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Expected {} bytes, got {}. File may be corrupt.", S::SIZE, buffer.len())
            ))
        }
        
        Ok(Self { data: buffer.into(), stage: PhantomData })
    }

    /// Save this lookup table to a file
    pub fn save(&self, file_path: &str) -> Result<(), std::io::Error> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(file_path)?;
        file.write_all(&self.data)?;
        Ok(())
    }

    /// Evaluate the minimum number of moves from this cube state to the goal
    pub fn eval(&self, cube: &Cube) -> u8 {
        self.data[S::indexer(cube)]
    }

    /// Compute lookup table using Breadth First Search (BFS)
    fn build_table_bfs() -> Box<[u8]> {
        let mut result = vec![u8::MAX; S::SIZE];
        let mut queue = VecDeque::new();
        
        // Start from solved cube
        let solved = Cube::new();
        result[S::indexer(&solved)] = 0;
        queue.push_back((solved, 0));
        
        let mut num_items = 1;
        let mut last_depth = 0;
        
        while let Some((cube, depth)) = queue.pop_front() {
            if depth > last_depth {
                last_depth = depth;
                println!("  Depth {}: {} states found", depth, num_items);
            }
            
            for turn in S::MOVE_POOL.iter() {
                let mut child = cube.clone();
                child.turn(turn);
                let index = S::indexer(&child);
                
                if result[index] == u8::MAX {  // Not visited yet
                    result[index] = depth + 1;
                    num_items += 1;
                    queue.push_back((child, depth + 1));
                }
            }
        }
        
        println!("  Completed!");
        result.into()
    }
}