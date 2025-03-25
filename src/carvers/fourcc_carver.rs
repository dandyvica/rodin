// carves files ffollowing the FourCC pattern
use std::{
    fmt::{self, Debug},
    fs::{self, File},
    io::{BufWriter, Cursor, Write},
    path::Path,
};

use byteorder::{BigEndian, ReadBytesExt};
use hex_literal::hex;
use log::{debug, info};

use crate::filetypes::FileTypeCounter;

use super::CarvingResult;

// const CHUNK_IHDR: u32 = u32::from_ne_bytes([b'I', b'H', b'D', b'R']); // Image header (size, color type, etc.)
// const CHUNK_PLTE: u32 = u32::from_ne_bytes([b'P', b'L', b'T', b'E']); // Palette table (for indexed-color images)
// const CHUNK_IDAT: u32 = u32::from_ne_bytes([b'I', b'D', b'A', b'T']); // Image data (compressed with zlib/deflate)
// const CHUNK_IEND: u32 = u32::from_ne_bytes([b'I', b'E', b'N', b'D']); // Marks the end of the PNG file
// const CHUNK_tEXt: u32 = u32::from_ne_bytes([b't', b'E', b'X', b't']); // Plain text metadata (e.g., author, title)
// const CHUNK_iTXt: u32 = u32::from_ne_bytes([b'i', b'T', b'X', b't']); // International text (UTF-8 encoded text)
// const CHUNK_tIME: u32 = u32::from_ne_bytes([b't', b'I', b'M', b'E']); // Last modification time of the image
// const CHUNK_gAMA: u32 = u32::from_ne_bytes([b'g', b'A', b'M', b'A']); // Gamma correction information

#[derive(Debug, Default)]
pub struct Chunk {
    length: u32,         // Length of the chunk data (big-endian)
    chunk_type: [u8; 4], // ASCII letters defining the chunk type
    chunk_data: Vec<u8>, // Actual image data or metadata
    crc: u32,            // Cyclic Redundancy Check for integrity.
}

// #[derive(Debug, Default)]
// #[repr(u32)]
// pub enum ChunkType {
//     #[default]
//     IHDR = CHUNK_IHDR,
//     PLTE = CHUNK_PLTE,
//     IDAT = CHUNK_IDAT,
//     IEND = CHUNK_IEND,
//     tEXt = CHUNK_tEXt,
//     iTXt = CHUNK_iTXt,
//     tIME = CHUNK_tIME,
//     gAMA = CHUNK_gAMA,
// }

impl Chunk {
    fn from_bytes(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()> {
        self.length = buffer.read_u32::<BigEndian>()?;
        self.chunk_type[0] = buffer.read_u8()?;
        self.chunk_type[1] = buffer.read_u8()?;
        self.chunk_type[2] = buffer.read_u8()?;
        self.chunk_type[3] = buffer.read_u8()?;

        // skip data for data and CRC
        let pos = buffer.position();
        buffer.set_position(pos + self.length as u64 + 4);

        Ok(())
    }
}

impl fmt::Display for Chunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "length={} ", self.length)?;
        write!(
            f,
            "chunk={}",
            String::from_utf8(self.chunk_type.to_vec()).unwrap()
        )
    }
}

pub fn fourcc_carver(
    mmap: &[u8],
    ft_counter: &mut FileTypeCounter,
    category: &str,
    min_size: usize,
) -> anyhow::Result<CarvingResult> {
    // read signature
    let buf: [u8; 8] = mmap[..8].try_into().expect("foo");
    assert_eq!(buf, hex!("89 50 4E 47 0D 0A 1A 0A"));

    // read all chunks till the end
    let mut cursor = Cursor::new(mmap);
    cursor.set_position(8);

    loop {
        let position = cursor.position();
        let mut chunk = Chunk::default();
        chunk.from_bytes(&mut cursor)?;
        println!("chunk={}, position={}", chunk, position);

        if chunk.chunk_type == *b"IEND" {
            // we found the end of PNG
            break;
        }
    }
    println!("IEND found!!");

    // the cursor position is now the end of file
    let payload = &mmap[..cursor.position() as usize];

    // save bitmap
    let index = {
        let map = ft_counter.lock().unwrap();
        map["png"]
        // format!("{}_{:08}.{}", ext, map[&ext], ext)
    };

    // test sub-directory for category: check if the directory exists
    if !Path::new(category).exists() {
        // create the directory including subdir
        fs::create_dir_all(category).expect("Failed to create directory");
    }

    // now we can build file name
    let file_name = format!("{}/{}_{:08}.{}", category, "png", index, "png");

    let file = File::create(&file_name)?;
    let mut writer = BufWriter::new(file);

    writer.write_all(&payload)?;
    writer.flush()?; // Ensure everything is written        

    // add 1 to our per extension counter
    let mut map = ft_counter.lock().unwrap();
    map.entry(String::from("png"))
        .and_modify(|count| *count += 1);

    // move offset, lock will be automatically released
    return Ok(CarvingResult::new(cursor.position(), &file_name));
}
