// all the carvers are located as modules here
use std::{
    fmt::Debug,
    fs::{self, File},
    io::{BufWriter, Write},
    path::Path,
};

use crate::filetypes::{Corpus, FileTypeCounter};

// this is returned by the main seach function
#[derive(Debug, Default)]
pub struct CarvingResult {
    // if a file is found, we need to move the offset when saving that file
    pub offset: u64,

    // if a file is found, this is its file name
    pub file_name: Option<String>,
}

impl CarvingResult {
    // helper to define a new result
    pub fn new(offset: u64, file_name: &str) -> Self {
        Self {
            offset: offset.into(),
            file_name: Some(String::from(file_name)),
        }
    }
}

// helper function to save the carved file
// pub fn save_file(
//     payload: &[u8],
//     ft_counter: &mut FileTypeCounter,
//     corpus: &Corpus,
// ) -> anyhow::Result<()> {
//     // save file
//     let ext = header.ext();
//     let index = {
//         let map = ft_counter.lock().unwrap();
//         map[&ext]
//     };

//     // test sub-directory for category: check if the directory exists
//     if !Path::new(category).exists() {
//         // create the directory including subdir
//         fs::create_dir_all(category).expect("Failed to create directory");
//     }

//     // now we can build file name
//     let file_name = format!("{}/{}_{:08}.{}", category, ext, index, ext);

//     let file = File::create(&file_name)?;
//     let mut writer = BufWriter::new(file);

//     writer.write_all(&payload)?;
//     writer.flush()?; // Ensure everything is written

//     // add 1 to our per extension counter
//     let mut map = ft_counter.lock().unwrap();
//     map.entry(ext).and_modify(|count| *count += 1);

//     Ok(())
// }

// carve when the file header contains the file size
pub mod fourcc_carver;
pub mod size_carver;
