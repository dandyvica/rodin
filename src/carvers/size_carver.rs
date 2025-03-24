use std::{
    fmt::Debug,
    fs::{self, File},
    io::{BufWriter, Cursor, Write},
    path::Path,
};

use log::debug;

use crate::filetypes::FileTypeCounter;

use super::CarvingResult;

pub trait SizeCarver {
    fn size(&self) -> usize; // size of the file we're trying to carve
    fn is_genuine(&self) -> bool; // true to guess whether what we're carving out could be a genuine file
    fn ext(&self) -> String; // the file extension of what we're trying to carve
    fn from_bytes(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()>; // copy bytes to structure
}

pub fn carve_using_size<T>(
    buffer: &[u8],
    ft_counter: &mut FileTypeCounter,
    category: &str,
    min_size: usize,
) -> anyhow::Result<CarvingResult>
where
    T: SizeCarver + Default + Debug,
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
            return Ok(CarvingResult::default());
        }

        // payload will receive all data
        let payload = &buffer[..header.size()];

        // save bitmap
        let ext = header.ext();
        let index = {
            let map = ft_counter.lock().unwrap();
            map[&ext]
            // format!("{}_{:08}.{}", ext, map[&ext], ext)
        };

        // test sub-directory for category: check if the directory exists
        if !Path::new(category).exists() {
            // create the directory including subdir
            fs::create_dir_all(category).expect("Failed to create directory");
        }

        // now we can build file name
        let file_name = format!("{}/{}_{:08}.{}", category, ext, index, ext);

        let file = File::create(&file_name)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(&payload)?;
        writer.flush()?; // Ensure everything is written        

        // add 1 to our per extension counter
        let mut map = ft_counter.lock().unwrap();
        map.entry(ext).and_modify(|count| *count += 1);

        // move offset, lock will be automatically released
        return Ok(CarvingResult::new(header.size() as u64, &file_name));
    } else {
        return Ok(CarvingResult::default());
    }
}
// pub trait SizeCarver {
//     fn size(&self) -> usize; // size of the file we're trying to carve
//     fn is_genuine(&self) -> bool; // true to guess whether what we're carving out could be a genuine file
//     fn ext(&self) -> String; // the file extension of what we're trying to carve
//     fn from_bytes(&mut self, buffer: &mut std::io::Cursor<&[u8]>) -> std::io::Result<()>; // copy bytes to structure
// }

// pub fn carve_using_size<T>(
//     buffer: &[u8],
//     ft_counter: &mut FileTypeCounter,
//     min_size: usize,
// ) -> anyhow::Result<CarvingResult>
// where
//     T: SizeCarver + Default + Debug,
// {
//     // read magic
//     let mut header = T::default();
//     let mut cursor = Cursor::new(buffer);
//     header.from_bytes(&mut cursor)?;
//     debug!("header={:?}", header);

//     // now read remaining data if this appears to be a real file
//     if header.is_genuine() {
//         // file is to small, so forget
//         if header.size() < min_size {
//             return Ok(CarvingResult::default());
//         }

//         // payload will receive all data
//         let payload = &buffer[..header.size()];

//         // save bitmap
//         let ext = header.ext();
//         let file_name = {
//             let map = ft_counter.lock().unwrap();
//             format!("{}_{:08}.{}", ext, map[&ext], ext)
//         };
//         let file = File::create(&file_name)?;
//         let mut writer = BufWriter::new(file);

//         writer.write_all(&payload)?;
//         writer.flush()?; // Ensure everything is written

//         // add 1 to our per extension counter
//         let mut map = ft_counter.lock().unwrap();
//         map.entry(ext).and_modify(|count| *count += 1);

//         // move offset, lock will be automatically released
//         return Ok(CarvingResult::new(header.size() as u64, &file_name));
//     } else {
//         return Ok(CarvingResult::default());
//     }
// }
