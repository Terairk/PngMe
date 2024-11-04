use crc::{Crc, CRC_32_ISO_HDLC};
use std::fmt::{Display, Formatter};
use crate::chunk_type::ChunkType;
use anyhow::{anyhow, bail, Context, Result};
use thiserror::Error;

static CRC_ALGO: Crc<u32> = Crc::<u32>::new(&CRC_32_ISO_HDLC);

pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    chunk_data: Vec<u8>,
    crc: u32,
}

#[derive(Error, Debug)]
enum ChunkError {
    #[error("Chunk length {0} is too large. It should not exceed 2^31 bytes")]
    LengthTooLarge(u32),
    #[error("Chunk length {0} is incorrect (too large) for the chunk data")]
    IncorrectLength(u32),
    #[error("Crc mismatch. Expected: {expected}, Calculated: {calculated}")]
    InvalidCrc{
        expected: u32,
        calculated: u32,
    },
}

impl TryFrom<&[u8]> for Chunk {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self> {
        let length = u32::from_be_bytes(
            value.get(0..4)
                .ok_or(anyhow!("Input slice is too short for chunk length"))?
                .try_into()
                .context("Failed to convert into integer from 4 bytes")?);
        
        if 2u32.pow(31) < length {
            bail!(ChunkError::LengthTooLarge(length));
        }

        let chunk_type_array: [u8; 4] = value.get(4..8)
            .ok_or(anyhow!("Input slice is too short, not of size 8 for chunk type"))?
            .try_into()?;
        let chunk_type = ChunkType::try_from(chunk_type_array)?;
        let chunk_data = value.get(8..8 + length as usize).ok_or(ChunkError::IncorrectLength(length))?;
        
        let crc = u32::from_be_bytes(value[8 + length as usize..].try_into()
            .context("Failed to convert into integer from 4 bytes for crc")?);

        // returns an error if it occurs
        Self::validate_crc(crc, chunk_data, &chunk_type.bytes())?;

        Ok(Chunk {
            length,
            chunk_type,
            chunk_data: Vec::from(chunk_data),
            crc,
        })
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Chunk length: {}", self.length)?;
        writeln!(f, "Chunk type: {}", self.chunk_type)?;
        writeln!(f, "CRC: {}", self.crc)
    }
}

impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Chunk {
        let length = data.len() as u32;

        let mut digest = CRC_ALGO.digest();
        digest.update(&chunk_type.bytes()[..]);
        digest.update(data.as_slice());
        let crc = digest.finalize();

        Chunk {
            length,
            chunk_type,
            chunk_data: data,
            crc
        }
    }
    
    pub fn length(&self) -> u32 {
        self.length
    }
    
    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }
    
    fn data(&self) -> &[u8] {
        self.chunk_data.as_slice()
    }
    
    fn crc(&self) -> u32 {
        self.crc
    }
    
    pub fn data_as_string(&self) -> Result<String> {
        std::str::from_utf8(&self.chunk_data)
            .map(String::from)
            .map_err(|e| anyhow!("UTF-8 conversion error: {e}"))
    }
    
    pub fn as_bytes(&self) -> Vec<u8> {
        let capacity = 4 + 4 + self.data().len() + 4;
        let mut result = Vec::with_capacity(capacity);
        result.extend_from_slice(&self.length.to_be_bytes());
        result.extend_from_slice(&self.chunk_type.bytes());
        result.extend_from_slice(self.data());
        result.extend_from_slice(&self.crc().to_be_bytes());
        result
    }

    fn validate_crc(crc: u32, data: &[u8], chunk_type: &[u8; 4]) -> Result<()> {
        let mut digest = CRC_ALGO.digest();
        digest.update(chunk_type);
        digest.update(data);
        let calculated_crc = digest.finalize();
        if crc == calculated_crc {
            Ok(())
        } else {
            Err(ChunkError::InvalidCrc {
                expected: crc,
                calculated: calculated_crc,
            })?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk_type::ChunkType;
    use std::str::FromStr;

    fn testing_chunk() -> Chunk {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        Chunk::try_from(chunk_data.as_ref()).unwrap()
    }

    #[test]
    fn test_new_chunk() {
        let chunk_type = ChunkType::from_str("RuSt").unwrap();
        let data = "This is where your secret message will be!".as_bytes().to_vec();
        let chunk = Chunk::new(chunk_type, data);
        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_chunk_length() {
        let chunk = testing_chunk();
        assert_eq!(chunk.length(), 42);
    }

    #[test]
    fn test_chunk_type() {
        let chunk = testing_chunk();
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
    }

    #[test]
    fn test_chunk_string() {
        let chunk = testing_chunk();
        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");
        assert_eq!(chunk_string, expected_chunk_string);
    }

    #[test]
    fn test_chunk_crc() {
        let chunk = testing_chunk();
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_valid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref()).unwrap();

        let chunk_string = chunk.data_as_string().unwrap();
        let expected_chunk_string = String::from("This is where your secret message will be!");

        assert_eq!(chunk.length(), 42);
        assert_eq!(chunk.chunk_type().to_string(), String::from("RuSt"));
        assert_eq!(chunk_string, expected_chunk_string);
        assert_eq!(chunk.crc(), 2882656334);
    }

    #[test]
    fn test_invalid_chunk_from_bytes() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656333;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk = Chunk::try_from(chunk_data.as_ref());

        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_trait_impls() {
        let data_length: u32 = 42;
        let chunk_type = "RuSt".as_bytes();
        let message_bytes = "This is where your secret message will be!".as_bytes();
        let crc: u32 = 2882656334;

        let chunk_data: Vec<u8> = data_length
            .to_be_bytes()
            .iter()
            .chain(chunk_type.iter())
            .chain(message_bytes.iter())
            .chain(crc.to_be_bytes().iter())
            .copied()
            .collect();

        let chunk: Chunk = TryFrom::try_from(chunk_data.as_ref()).unwrap();

        let _chunk_string = format!("{}", chunk);
    }
}
