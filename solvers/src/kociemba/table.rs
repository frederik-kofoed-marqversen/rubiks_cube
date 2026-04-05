use std::io::{Read, Write};

const HEADER_SIZE: usize = 12; // 4 magic + 4 idx1_width + 4 idx2_width
const MAGIC_NUMBER: u32 = 0x4C544432; // "LTD2" = LookupTable2D in ASCII

pub struct LookupTable2D<T: Copy, const IDX2_WIDTH: usize> {
    pub data: Vec<T>,
}

impl<T: Copy, const IDX2_WIDTH: usize> LookupTable2D<T, IDX2_WIDTH> {
    #[inline]
    pub fn new(idx1_width: usize, default: T) -> Self {
        Self {
            data: vec![default; idx1_width * IDX2_WIDTH],
        }
    }

    pub fn idx1_width(&self) -> usize {
        self.data.len() / IDX2_WIDTH
    }

    pub fn idx2_width(&self) -> usize {
        IDX2_WIDTH
    }

    // #[inline]
    // pub fn get(&self, idx1: usize, idx2: usize) -> T {
    //     self.data[idx1 * IDX2_WIDTH + idx2]
    // }

    // #[inline]
    // pub fn set(&mut self, idx1: usize, idx2: usize, val: T) {
    //     self.data[idx1 * IDX2_WIDTH + idx2] = val;
    // }

    pub fn serialize_to_writer<W: Write>(&self, writer: &mut W) -> Result<(), std::io::Error> {
        // Write header
        writer.write_all(&MAGIC_NUMBER.to_le_bytes())?;
        writer.write_all(&(self.idx1_width() as u32).to_le_bytes())?;
        writer.write_all(&(IDX2_WIDTH as u32).to_le_bytes())?;

        // Write data
        let data_bytes = unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const u8,
                self.data.len() * std::mem::size_of::<T>(),
            )
        };
        writer.write_all(data_bytes)?;
        Ok(())
    }

    pub fn deserialize_from_reader<R: Read>(reader: &mut R) -> Result<Self, std::io::Error> {
        // Read header
        let mut header = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header)?;

        let magic = u32::from_le_bytes([header[0], header[1], header[2], header[3]]);
        let idx1_width = u32::from_le_bytes([header[4], header[5], header[6], header[7]]) as usize;
        let idx2_width =
            u32::from_le_bytes([header[8], header[9], header[10], header[11]]) as usize;

        // Validate header
        if magic != MAGIC_NUMBER {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid data."),
            ));
        }

        // Read data
        let data_size = idx1_width * idx2_width;
        let byte_size = data_size * std::mem::size_of::<T>();
        let mut bytes = vec![0u8; byte_size];
        reader.read_exact(&mut bytes)?;

        // Convert bytes to T
        let mut data = Vec::with_capacity(data_size);
        unsafe {
            let t_ptr = bytes.as_ptr() as *const T;
            for i in 0..data_size {
                data.push(*t_ptr.add(i));
            }
        }

        Ok(Self { data })
    }
}
