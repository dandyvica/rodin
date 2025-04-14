// carves files ffollowing the FourCC pattern
use std::{fmt::Debug, io::Cursor};

use log::{debug, trace};
use std::io::ErrorKind;

use crate::{
    deserializer::Deserializer,
    filetypes::corpus::{CarvingMethod, FileType},
};

use super::CarvingResult;

pub trait FourCCCarver {
    fn is_end(&self) -> bool; // test whether we met the final chunk
}

pub fn fourcc_carver<T, U>(mmap: &[u8], ft: &FileType) -> anyhow::Result<CarvingResult>
where
    T: Debug + Default + Deserializer,
    U: Debug + Default + FourCCCarver + Deserializer,
{
    // println!("==========================");
    // read magic
    let mut header = T::default();
    let mut cursor = Cursor::new(mmap);
    header.deserialize(&mut cursor)?;

    loop {
        let mut chunk = U::default();

        match chunk.deserialize(&mut cursor) {
            Ok(_) => {
                // did we find the end marker ?
                if chunk.is_end() {
                    trace!(
                        "file type {}: EOF marker found!! = {:x?} at offset: {:X?}",
                        &ft.ext,
                        chunk,
                        cursor.position()
                    );
                    break;
                }
            }
            Err(e) => match e.kind() {
                // not really an I/O error
                // depending on the carving method, we stop here or continue
                ErrorKind::InvalidData => match ft.carving_method {
                    // halt whenever a chunk is not recognized
                    CarvingMethod::Strict => return Ok(CarvingResult::default()),

                    // we continue till marker end or maximum length reached
                    CarvingMethod::Simple => (),

                    // we stop here
                    CarvingMethod::Fancy => return Ok(CarvingResult::default()),
                },
                // here, true I/O error
                _ => {
                    debug!(
                        "file type {}: I/O error: {} deserializing chunk={:?}, skipping",
                        &ft.ext, e, chunk
                    );
                    return Ok(CarvingResult::default());
                }
            },
        }
    }

    // the cursor position is now the end of file
    let payload = &mmap[..cursor.position() as usize];

    // if the file we found is not bug enough, do not consider it
    if payload.len() < ft.min_size {
        return Ok(CarvingResult::default());
    }

    // save file
    let file_name = ft.save_file(payload)?;

    // move offset, lock will be automatically released
    Ok(CarvingResult::new(cursor.position(), &file_name))
}
