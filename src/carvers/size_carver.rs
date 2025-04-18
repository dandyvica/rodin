use std::{fmt::Debug, io::Cursor};

use log::debug;

use crate::{deserializer::Deserializer, filetypes::corpus::FileType};

use super::CarvingResult;

pub trait SizeCarver {
    fn size(&self) -> usize; // size of the file we're trying to carve
    fn is_genuine(&self) -> bool; // true to guess whether what we're carving out could be a genuine file
    fn ext(&self) -> String; // the file extension of what we're trying to carve
}

pub fn carve_using_size<T>(mmap: &[u8], ft: &FileType) -> anyhow::Result<CarvingResult>
where
    T: SizeCarver + Deserializer + Default + Debug,
{
    // read magic
    let mut header = T::default();
    let mut cursor = Cursor::new(mmap);
    header.deserialize(&mut cursor)?;
    debug!("header={:?}", header);

    // now read remaining data if this appears to be a real file
    if header.is_genuine() {
        // file is to small, so forget
        if header.size() < ft.min_size {
            return Ok(CarvingResult::default());
        }

        // payload will receive all data
        let payload = &mmap[..header.size()];

        // save file
        let file_name = ft.save_file(payload)?;

        // move offset, lock will be automatically released
        Ok(CarvingResult::new(header.size() as u64, &file_name, payload.len()))
    } else {
        Ok(CarvingResult::default())
    }
}
