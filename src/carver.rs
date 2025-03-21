use std::{
    collections::HashMap,
    fs::File,
    io::{BufWriter, Cursor, Write},
};

use log::debug;

pub trait Carver {
    fn size(&self) -> usize; // size of the file we're trying to carve
    fn is_genuine(&self) -> bool; // true to guess whether what we're carving out could be a genuine file
    fn ext(&self) -> String; // the file extension of what we're trying to carve
    fn from_bytes(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()>; // copy bytes to structure
}

pub fn carve_using_size<T>(
    buffer: &[u8],
    index: &mut HashMap<String, u32>,
    min_size: usize,
) -> anyhow::Result<u64>
where
    T: Carver + Default + std::fmt::Debug,
{
    // read magic
    let mut header = T::default();
    let mut cursor = Cursor::new(buffer);
    header.from_bytes(&mut cursor)?;
    debug!("header={:?}", header);

    // now read remaining data if this appears to be a real file
    if header.is_genuine() {
        // file is to small, so forget
        if header.size() < min_size {
            return Ok(0);
        }

        // payload will receive all data
        let payload = &buffer[..header.size()];

        // save bitmap
        let ext = header.ext();
        let filename = format!("{}_{:08}.{}", ext, index[&ext], ext);
        let file = File::create(&filename)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(&payload)?;
        writer.flush()?; // Ensure everything is written        

        // add 1 to our per extension index
        index.entry(ext).and_modify(|count| *count += 1);

        // move offset
        return Ok(header.size() as u64);
    } else {
        return Ok(0);
    }
}
