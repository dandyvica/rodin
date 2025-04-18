use core::fmt;
use std::{
    io::{Cursor, Error, ErrorKind, Read},
    ops::Deref,
};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{carvers::fourcc_carver::FourCCCarver, deserializer::Deserializer, err};

#[derive(Debug, Default)]
pub struct PNGHeader {
    signature: u64,
}

impl Deserializer for PNGHeader {
    fn deserialize(&mut self, buffer: &mut Cursor<&[u8]>) -> std::io::Result<usize> {
        self.signature = buffer.read_u64::<BigEndian>()?;

        Ok(4)
    }
}

// a chunk type is given by just 4 bytes
#[derive(Debug, Default)]
pub struct ChunkType([u8; 4]);

impl ChunkType {
    // not all arrays of 2 bytes are valid Jpeg segments
    fn is_valid(&self) -> bool {
        if self.0 != *b"IHDR"
            && self.0 != *b"PLTE"
            && self.0 != *b"IDAT"
            && self.0 != *b"IEND"
            && self.0 != *b"tEXt"
            && self.0 != *b"iTXt"
            && self.0 != *b"tIME"
            && self.0 != *b"gAMA"
            && self.0 != *b"sRGB"
            && self.0 != *b"iCCP"
            && self.0 != *b"pHYs"
        {
            false
        } else {
            true
        }
    }
}

impl Deref for ChunkType {
    type Target = [u8; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PartialEq<[u8; 4]> for ChunkType {
    fn eq(&self, other: &[u8; 4]) -> bool {
        self.0 == *other
    }
}

impl Deserializer for ChunkType {
    fn deserialize(&mut self, buffer: &mut Cursor<&[u8]>) -> std::io::Result<usize> {
        buffer.read_exact(self.0.as_mut_slice())?;

        Ok(4)
    }
}

#[derive(Default)]
pub struct PNGChunk {
    length: u32,           // Length of the chunk data (big-endian)
    chunk_type: ChunkType, // ASCII letters defining the chunk type
}

impl fmt::Debug for PNGChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "length={} ", self.length)?;
        write!(
            f,
            "chunk={}",
            String::from_utf8(self.chunk_type.to_vec()).unwrap()
        )
    }
}

impl Deserializer for PNGChunk {
    fn deserialize(&mut self, buffer: &mut Cursor<&[u8]>) -> std::io::Result<usize> {
        self.length = buffer.read_u32::<BigEndian>()?;
        let _ = self.chunk_type.deserialize(buffer)?;

        // compare against usual chunk types
        if !self.chunk_type.is_valid() {
            return err!(ErrorKind::InvalidData);
        }

        // skip data for data and CRC
        let pos = buffer.position();
        buffer.set_position(pos + self.length as u64 + 4);

        Ok(pos as usize + self.length as usize + 4)
    }
}

impl FourCCCarver for PNGChunk {
    fn is_end(&self) -> bool {
        self.chunk_type == *b"IEND"
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn chunk_type() {
        let ct = ChunkType(*b"ABCD");
        assert!(!ct.is_valid());

        let ct = ChunkType(*b"IHDR");
        assert!(ct.is_valid());

        let v = Vec::from(*b"IHDR");
        let mut c = Cursor::new(v.as_slice());
        let mut ct = ChunkType::default();
        let n = ct.deserialize(&mut c).unwrap();
        assert_eq!(n, 4);
        assert!(ct.is_valid());
        assert_eq!(&ct, b"IHDR");
    }

    #[test]
    fn chunk() {
        let raw_data = hex!("00 00 00  01 73 52 47  42 00 AE CE  1C E9");
        let mut c = Cursor::new(raw_data.as_slice());
        let mut chunk = PNGChunk::default();
        let n = chunk.deserialize(&mut c).unwrap();
        assert_eq!(n, 13);

        assert_eq!(chunk.length, 1);
        assert_eq!(&chunk.chunk_type, b"sRGB");
    }
}
