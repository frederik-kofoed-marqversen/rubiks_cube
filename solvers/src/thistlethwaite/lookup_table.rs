use super::stages::Stage;
use super::cube::Cube;
use std::io::{Read, Write};
use std::marker::PhantomData;

pub struct LookupTable<S> {
    pub data: Box<[u8]>,
    stage: PhantomData<S>
}

impl<'a, S: Stage<'a>> LookupTable<S> {
    pub fn new(data_file: Option<&str>) -> Self {
        match data_file {
            None => {
                return Self {data: Self::build_table(), stage: PhantomData}
            },
            Some(file_path) => {
                match Self::load_data_from_file(file_path) {
                    Ok(data) => return Self {data: data, stage: PhantomData},
                    Err(_) => {
                        let table = Self::new(None);
                        table.save_data_to_file(file_path).unwrap();
                        return table
                    }
                }
            }
        }
    }

    pub fn eval(&self, cube: &Cube) -> u8 {
        self.data[S::indexer(cube)]
    }

    fn load_data_from_file(file_path: &str) -> Result<Box<[u8]>, std::io::Error> {
        // load from file
        let mut file = std::fs::File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if buffer.len() != S::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Save file seems to be corrupt."
            ))
        } else {
            return Ok(buffer.into())
        }
    }

    pub fn save_data_to_file(&self, file_path: &str) -> Result<(), std::io::Error> {
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
            dbg!(&depth_limit, S::SIZE - num_items);

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
        return result.into()
    }
}

/* // Alternative implementations using a constant filepath for storing binary
impl<'a, S: Stage<'a>> LookupTable<S> {
    pub fn new() -> Self {
        match Self::load_data_from_file() {
            Ok(data) => return Self {data: data, stage: PhantomData},
            Err(_) => {
                let data = Self::build_table();
                let table = Self {data: data, stage: PhantomData};
                table.save_data_to_file().unwrap();
                return table
            }
        }
    }

    fn load_data_from_file() -> Result<Box<[u8]>, std::io::Error> {
        // load from file
        let mut file = std::fs::File::open(S::FILEPATH)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        if buffer.len() != S::SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Save file seems to be corrupt."
            ))
        } else {
            return Ok(buffer.into())
        }
    }

    fn save_data_to_file(&self) -> Result<(), std::io::Error> {
        let mut file = std::fs::File::create(S::FILEPATH)?;
        file.write_all(&self.data)?;
        Ok(())
    }
} */