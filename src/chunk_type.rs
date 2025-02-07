use std::fmt::{Display, Formatter};
use std::str::FromStr;
use anyhow::{bail, Context, Result};
use thiserror::Error;

#[derive(PartialEq, Eq, Debug)]
pub struct ChunkType {
    data: [u8; 4],
}

#[derive(Debug, Error)]
enum ChunkTypeError {
    #[error("String is not ASCII")]
    NonAscii,
    #[error("String is not 4 (actual length: {0})")]
    InvalidLength(usize),
    #[error("String contains non-ASCII letters")]
    NonAlphabetic,
    
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = anyhow::Error;

    fn try_from(value: [u8; 4]) -> Result<Self, Self::Error> {
        Ok(ChunkType { data: value })
    }
}

impl FromStr for ChunkType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut errors = Vec::new();
        
        if !s.is_ascii() {
            errors.push(ChunkTypeError::NonAscii);
        }
        
        if s.len() != 4 {
            errors.push(ChunkTypeError::InvalidLength(s.len()));
        }
        
        if !s.chars().all(|c| c.is_ascii_alphabetic()) {
            errors.push(ChunkTypeError::NonAlphabetic);
        }
        
        if !errors.is_empty() {
            let error_message = errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            bail!("Invalid chunk type: {error_message}")
        }
        
        Ok(ChunkType {
            data: s.as_bytes().try_into().context("Failed to convert string to bytes")?
        })
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", std::str::from_utf8(&self.data).unwrap())
    }
}

impl ChunkType {
    pub fn length() -> u32 {
        4
    }
    
    pub fn bytes(&self) -> [u8; 4] {
        self.data
    }
    
    fn is_valid(&self) -> bool {
        self.data[2].is_ascii_uppercase()
    }
    
    fn is_critical(&self) -> bool {
        self.data[0].is_ascii_uppercase()
    }
    
    fn is_public(&self) -> bool {
        self.data[1].is_ascii_uppercase()
    }
    
    fn is_reserved_bit_valid(&self) -> bool {
        self.data[2].is_ascii_uppercase()
    }
    
    fn is_safe_to_copy(&self) -> bool {
        self.data[3].is_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;
    use std::str::FromStr;

    #[test]
    pub fn test_chunk_type_from_bytes() {
        let expected = [82, 117, 83, 116];
        let actual = ChunkType::try_from([82, 117, 83, 116]).unwrap();

        assert_eq!(expected, actual.bytes());
    }

    #[test]
    pub fn test_chunk_type_from_str() {
        let expected = ChunkType::try_from([82, 117, 83, 116]).unwrap();
        let actual = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    pub fn test_chunk_type_is_critical() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_not_critical() {
        let chunk = ChunkType::from_str("ruSt").unwrap();
        assert!(!chunk.is_critical());
    }

    #[test]
    pub fn test_chunk_type_is_public() {
        let chunk = ChunkType::from_str("RUSt").unwrap();
        assert!(chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_not_public() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(!chunk.is_public());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_reserved_bit_invalid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_reserved_bit_valid());
    }

    #[test]
    pub fn test_chunk_type_is_safe_to_copy() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_chunk_type_is_unsafe_to_copy() {
        let chunk = ChunkType::from_str("RuST").unwrap();
        assert!(!chunk.is_safe_to_copy());
    }

    #[test]
    pub fn test_valid_chunk_is_valid() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert!(chunk.is_valid());
    }

    #[test]
    pub fn test_invalid_chunk_is_valid() {
        let chunk = ChunkType::from_str("Rust").unwrap();
        assert!(!chunk.is_valid());

        let chunk = ChunkType::from_str("Ru1t");
        assert!(chunk.is_err());
    }

    #[test]
    pub fn test_chunk_type_string() {
        let chunk = ChunkType::from_str("RuSt").unwrap();
        assert_eq!(&chunk.to_string(), "RuSt");
    }

    #[test]
    pub fn test_chunk_type_trait_impls() {
        let chunk_type_1: ChunkType = TryFrom::try_from([82, 117, 83, 116]).unwrap();
        let chunk_type_2: ChunkType = FromStr::from_str("RuSt").unwrap();
        let _chunk_string = format!("{}", chunk_type_1);
        let _are_chunks_equal = chunk_type_1 == chunk_type_2;
    }
}
