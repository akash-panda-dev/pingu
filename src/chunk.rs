use std::fmt::Display;

use thiserror::Error;

use crate::chunk_type::{ChunkType, ChunkTypeErr};

#[derive(Debug, Error)]
pub enum ChunkError {
    #[error("Invalid chunk lenght: {0}")]
    InvalidLength(usize),
    #[error(transparent)]
    ConversionError(#[from] Box<dyn std::error::Error>),
    #[error(transparent)]
    InvalidChunkType(#[from] ChunkTypeErr),
    #[error("Invalid CRC")]
    InvalidCrc,
}

pub struct Chunk {
    length: u32,
    chunk_type: ChunkType,
    data: Vec<u8>,
    crc: u32,
}

#[allow(unused_variables, dead_code)]
impl Chunk {
    pub fn new(chunk_type: ChunkType, data: Vec<u8>) -> Self {
        let length = data.len() as u32;
        let mut bytes_to_checksum = vec![];
        bytes_to_checksum.extend_from_slice(&chunk_type.bytes());
        bytes_to_checksum.extend_from_slice(&data);

        let crc = crc32fast::hash(bytes_to_checksum.as_ref());

        Chunk {
            length,
            chunk_type,
            data,
            crc,
        }
    }

    fn length(&self) -> u32 {
        self.length
    }

    fn crc(&self) -> u32 {
        self.crc
    }

    pub fn chunk_type(&self) -> &ChunkType {
        &self.chunk_type
    }

    pub fn data_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.data.clone())
    }

    pub fn as_bytes(&self) -> Vec<u8> {
        let chunk_bytes: Vec<u8> = self.length.to_be_bytes()
        .iter()
        .chain(self.chunk_type.bytes().iter())
        .chain(self.data.iter())
        .chain(self.crc.to_be_bytes().iter())
        .copied()
        .collect();

        chunk_bytes
    }
}

impl TryFrom<&[u8]> for Chunk {
    type Error = ChunkError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < 12 {
            return Err(ChunkError::InvalidLength(value.len()));
        }

        let length = u32::from_be_bytes(
            value[0..4]
                .try_into()
                .map_err(|e| ChunkError::ConversionError(Box::new(e)))?,
        );
        let chunk_type_bytes: [u8; 4] = value[4..8]
            .try_into()
            .map_err(|e| ChunkError::ConversionError(Box::new(e)))?;
        let chunk_type = ChunkType::try_from(chunk_type_bytes)?;
        let data = value[8..(8 + length as usize)].to_vec();
        let crc = u32::from_be_bytes(
            value[8 + length as usize..]
                .try_into()
                .map_err(|e| ChunkError::ConversionError(Box::new(e)))?,
        );

        let mut bytes_to_checksum = vec![];
        bytes_to_checksum.extend_from_slice(&chunk_type_bytes);
        bytes_to_checksum.extend_from_slice(&data);

        if crc != crc32fast::hash(bytes_to_checksum.as_ref()) {
            return Err(ChunkError::InvalidCrc);
        }

        Ok(Chunk {
            length,
            chunk_type,
            data,
            crc,
        })
    }
}

impl Display for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Chunk Type: {}\nLength: {}\nData: {}\nCRC: {}",
            self.chunk_type,
            self.length,
            self.data_as_string().unwrap(),
            self.crc
        )
    }
}

#[allow(unused_variables)]
#[cfg(test)]
mod tests {
    use super::*;
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
        let data = "This is where your secret message will be!"
            .as_bytes()
            .to_vec();
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
