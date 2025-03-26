use core::fmt;
use std::io::{Error, ErrorKind};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{carvers::fourcc_carver::FourCCCarver, deserializer::Deserializer};

#[derive(Debug, Default)]
pub struct PNGHeader {
    signature: u64,
}

impl Deserializer for PNGHeader {
    fn deserialize(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()> {
        self.signature = buffer.read_u64::<BigEndian>()?;

        Ok(())
    }
}

#[derive(Default)]
pub struct PNGChunk {
    length: u32,         // Length of the chunk data (big-endian)
    chunk_type: [u8; 4], // ASCII letters defining the chunk type
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
    fn deserialize(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()> {
        self.length = buffer.read_u32::<BigEndian>()?;
        self.chunk_type[0] = buffer.read_u8()?;
        self.chunk_type[1] = buffer.read_u8()?;
        self.chunk_type[2] = buffer.read_u8()?;
        self.chunk_type[3] = buffer.read_u8()?;

        // compare against usual chunk types
        if self.chunk_type != *b"IHDR"
            && self.chunk_type != *b"PLTE"
            && self.chunk_type != *b"IDAT"
            && self.chunk_type != *b"IEND"
            && self.chunk_type != *b"tEXt"
            && self.chunk_type != *b"iTXt"
            && self.chunk_type != *b"tIME"
            && self.chunk_type != *b"gAMA"
            && self.chunk_type != *b"sRGB"
            && self.chunk_type != *b"iCCP"
            && self.chunk_type != *b"pHYs"
        {
            return Err(Error::from(ErrorKind::InvalidData));
        }

        // skip data for data and CRC
        let pos = buffer.position();
        buffer.set_position(pos + self.length as u64 + 4);

        Ok(())
    }
}

impl FourCCCarver for PNGChunk {
    fn is_end(&self) -> bool {
        self.chunk_type == *b"IEND"
    }
}
