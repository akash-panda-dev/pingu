use std::{
    fmt::Display,
    str::{self, FromStr},
};

use thiserror::Error;

#[derive(Debug, Error)]
enum ChunkTypeErr {
    #[error("Invalid byte: {0}, Reason: {1}")]
    InvalidByte(u8, String),
    #[error("Invalid chunk type: {0}")]
    InvalidChunkType(String),
}

#[derive(Debug, PartialEq)]
struct ChunkType {
    bytes: [u8; 4],
}

impl TryFrom<[u8; 4]> for ChunkType {
    type Error = ChunkTypeErr;

    fn try_from(bytes: [u8; 4]) -> Result<Self, Self::Error> {
        for byte in &bytes {
            if !byte.is_ascii_alphabetic() {
                return Err(ChunkTypeErr::InvalidByte(
                    *byte,
                    "byte is not a ascii alphabet".to_string(),
                ));
            }
        }

        Ok(ChunkType { bytes })
    }
}

impl FromStr for ChunkType {
    type Err = ChunkTypeErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 4 {
            return Err(ChunkTypeErr::InvalidChunkType(
                "length is not 4".to_string(),
            ));
        }

        let mut bytes: [u8; 4] = [0; 4];
        for (i, byte) in s.bytes().enumerate() {
            if !byte.is_ascii_alphabetic() {
                return Err(ChunkTypeErr::InvalidByte(
                    byte,
                    "byte is not a ascii alphabet".to_string(),
                ));
            }
            bytes[i] = byte;
        }
        Ok(ChunkType { bytes })
    }
}

impl Display for ChunkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", str::from_utf8(&self.bytes).map_err(|_| std::fmt::Error)?)
    }
}

impl ChunkType {
    fn bytes(&self) -> [u8; 4] {
        self.bytes
    }

    fn is_critical(&self) -> bool {
        self.bytes[0] & (1 << 5) == 0
    }

    fn is_public(&self) -> bool {
        self.bytes[1] & (1 << 5) == 0
    }

    fn is_reserved_bit_valid(&self) -> bool {
        self.bytes[2] & (1 << 5) == 0
    }

    fn is_safe_to_copy(&self) -> bool {
        self.bytes[3] & (1 << 5) != 0
    }

    fn is_valid(&self) -> bool {
        self.is_reserved_bit_valid()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

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
