// carves files ffollowing the FourCC pattern
use std::{fmt::Debug, io::Cursor};

use log::debug;

use crate::{deserializer::Deserializer, filetypes::corpus::FileType};

use super::CarvingResult;

pub trait FourCCCarver {
    fn is_end(&self) -> bool; // test whether we met the final chunk
}

pub fn fourcc_carver<T, U>(mmap: &[u8], ft: &FileType) -> anyhow::Result<CarvingResult>
where
    T: Debug + Default + Deserializer,
    U: Debug + Default + FourCCCarver + Deserializer,
{
    println!("==========================");
    // read magic
    let mut header = T::default();
    let mut cursor = Cursor::new(mmap);
    header.deserialize(&mut cursor)?;
    println!("header={:x?}", header);

    loop {
        let mut chunk = U::default();

        if let Err(_) = chunk.deserialize(&mut cursor) {
            println!("chunk={:?}", chunk);
            return Ok(CarvingResult::default());
        }
        // println!("chunk={}, position={}", chunk, position);

        if chunk.is_end() {
            // we found the end of fourCC file
            break;
        }
    }
    println!("IEND found!!");

    // the cursor position is now the end of file
    let payload = &mmap[..cursor.position() as usize];

    // save file
    let file_name = ft.save_file(payload)?;

    // move offset, lock will be automatically released
    Ok(CarvingResult::new(cursor.position(), &file_name))
}
