use super::stages::Stage;
use super::cube::Cube;
use std::io::{Read, Write};
use std::marker::PhantomData;

pub struct LookupTable<S> {
    data: Box<[u8]>,
    stage: PhantomData<S>
}

impl<'a, S: Stage<'a>> LookupTable<S> {
    /// Build a lookup table from scratch using IDDFS
    /// This is a slow operation but saves memory compared to BFS
    pub fn build() -> Self {
        println!("Building lookup table for stage (size: {})...", S::SIZE);
        Self {
            data: Self::build_table(),
            stage: PhantomData
        }
    }

    /// Load a precomputed lookup table from a file
    pub fn load(file_path: &str) -> Result<Self, std::io::Error> {
        let data = Self::load_data_from_file(file_path)?;
        Ok(Self { data, stage: PhantomData })
    }

    /// Save this lookup table to a file
    pub fn save(&self, file_path: &str) -> Result<(), std::io::Error> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        self.save_data_to_file(file_path)
    }

    /// Evaluate the minimum number of moves from this cube state to the goal
    pub fn eval(&self, cube: &Cube) -> u8 {
        self.data[S::indexer(cube)]
    }

    fn load_data_from_file(file_path: &str) -> Result<Box<[u8]>, std::io::Error> {
        let mut file = std::fs::File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if buffer.len() != S::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Expected {} bytes, got {}. File may be corrupt.", S::SIZE, buffer.len())
            ))
        }
        
        Ok(buffer.into())
    }

    fn save_data_to_file(&self, file_path: &str) -> Result<(), std::io::Error> {
        let mut file = std::fs::File::create(file_path)?;
        file.write_all(&self.data)?;
        Ok(())
    }

    /**Compute lookup table from scratch by Iterative Deepening Depth First Search (IDDFS). Although 
     * slow, this is done to save on memory since the width in a Breadth First Search would be large.
     */
    fn build_table() -> Box<[u8]> {
        // `result` will hold the minimum distance (number of turns) from a solved cube
        // to any configuration of edge orientations
        let mut result = vec![u8::MAX; S::SIZE];
        let mut num_items = 0;

        // Initialise the root
        result[S::indexer(&Cube::new())] = 0;
        num_items += 1;

        let mut depth_limit = 0;
        while num_items < S::SIZE {
            depth_limit += 1;
            println!("  Depth {}: {} states remaining", depth_limit, S::SIZE - num_items);

            let mut queue = vec![(Cube::new(), 0)];
            while let Some((parent, parent_depth)) = queue.pop() {
                for turn in S::MOVE_POOL.iter() {
                    let mut child = parent.clone();
                    child.turn(turn);
                    let index = S::indexer(&child);
                    let depth = parent_depth + 1;

                    if result[index] < depth {
                        // child can be reached at a shallower depth, so don't add it to the queue
                        continue;
                    }

                    if depth == depth_limit {
                        if result[index] > depth {
                            // This is the first time child is encountered, so record its depth
                            num_items += 1;
                            result[index] = depth;
                        }
                        // Child is at current depth limit, so don't add it to the queue
                        continue;
                    }

                    // Continue branch
                    queue.push((child, depth));
                }
            }
        }
        println!("  Completed!");
        result.into()
    }
}