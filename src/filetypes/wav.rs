use byteorder::{LittleEndian, ReadBytesExt};
use hex_literal::hex;

use crate::{carvers::size_carver::SizeCarver, deserializer::Deserializer};

// see: https://docs.fileformat.com/audio/wav/
#[derive(Debug, Default)]
pub struct WAV {
    magic: u32,      // should be "RIFF"
    size: u32,       // bitmap size in little endian
    id: u64,         // should be == "WAVEfmt "
    chunk_size: u32, // fmt chunk size
}

impl SizeCarver for WAV {
    fn size(&self) -> usize {
        self.size as usize + 8
    }

    fn is_genuine(&self) -> bool {
        const RIFF: [u8; 4] = hex!("52 49 46 46");
        const WAVE: [u8; 8] = hex!("57 41 56 45 66 6d 74 20");
        self.magic == u32::from_le_bytes(RIFF)
            && self.id == u64::from_le_bytes(WAVE)
            && self.chunk_size < 255
    }

    fn ext(&self) -> String {
        String::from("wav")
    }
}

impl Deserializer for WAV {
    fn deserialize(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<usize> {
        self.magic = buffer.read_u32::<LittleEndian>()?;
        self.size = buffer.read_u32::<LittleEndian>()?;
        self.id = buffer.read_u64::<LittleEndian>()?;
        self.chunk_size = buffer.read_u32::<LittleEndian>()?;

        Ok(20)
    }
}
