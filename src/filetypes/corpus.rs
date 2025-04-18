use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    ops::{Deref, Index},
    path::Path,
    sync::Mutex,
    usize,
};

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use hex_literal::hex;

use crate::{
    carvers::{CarvingResult, fourcc_carver::fourcc_carver, size_carver::carve_using_size},
    filetypes::{bmp::BMP, wav::WAV},
};

use super::{
    jpeg::JpegSegment,
    png::{PNGChunk, PNGHeader},
};

// alias for the carving function depending on the file type
pub type CarvingFunc = fn(&[u8], &FileType) -> anyhow::Result<CarvingResult>;

// carving mode
#[derive(Debug, Default)]
pub enum CarvingMethod {
    #[default]
    Strict, // respect the file structure
    Simple, // carve between a header and a footer
    Fancy,  // follows the file structure
}

// define what we're going to search for
#[derive(Debug)]
pub struct FileType {
    // the magic bytes to look for
    pub magic: Vec<u8>,

    // the file type extension
    pub ext: String,

    // the function used to carve
    pub carving_func: CarvingFunc,

    // category like images, audio, etc
    pub category: String,

    // the minimum file size to consider for this file type
    pub min_size: usize,

    // the maximum number of bytes we analyze when carving
    pub max_size: usize,

    // the current index of the file being carved
    pub index: Mutex<usize>,

    // the method used to carve
    pub carving_method: CarvingMethod,
}

impl FileType {
    // helper function to save the carved file
    pub fn save_file(&self, payload: &[u8]) -> anyhow::Result<String> {
        // test sub-directory for category: check if the directory exists
        if !Path::new(&self.category).exists() {
            // create the directory including subdir
            fs::create_dir_all(&self.category)?;
        }

        // now we can build file name
        let file_name = format!(
            "{}/{}_{:08}.{}",
            self.category,
            self.ext,
            self.index.lock().unwrap(),
            self.ext
        );

        let file = File::create(&file_name)?;
        let mut writer = BufWriter::new(file);

        writer.write_all(payload)?;
        writer.flush()?; // Ensure everything is written

        // add 1 to our per extension counter
        *self.index.lock().unwrap() += 1;

        Ok(file_name)
    }
}

// the list of all file types that we can carve
#[derive(Debug)]
pub struct Corpus(Vec<FileType>);

impl Corpus {
    // define all file types to carve
    // we had here all we know about a file type: magic number, how to carve it, its category, ...
    pub fn new(min_size: usize) -> Self {
        let mut vec = Vec::new();

        // BMP
        vec.push(FileType {
            magic: b"BM".to_vec(),
            ext: String::from("bmp"),
            carving_func: carve_using_size::<BMP>,
            category: String::from("images/bmp"),
            min_size: min_size,
            max_size: 1000000,
            index: Mutex::new(0),
            carving_method: CarvingMethod::Simple,
        });

        // WAV
        vec.push(FileType {
            magic: b"RIFF".to_vec(),
            ext: String::from("wav"),
            carving_func: carve_using_size::<WAV>,
            category: String::from("audio/wav"),
            min_size: min_size,
            max_size: 1000000,
            index: Mutex::new(0),
            carving_method: CarvingMethod::Simple,
        });

        // PNG
        vec.push(FileType {
            magic: hex!("89 50 4E 47 0D 0A 1A 0A").to_vec(),
            ext: String::from("png"),
            carving_func: fourcc_carver::<PNGHeader, PNGChunk>,
            category: String::from("images/png"),
            min_size: min_size,
            max_size: 1000000,
            index: Mutex::new(0),
            carving_method: CarvingMethod::Simple,
        });

        // JPEG
        vec.push(FileType {
            magic: hex!("FF D8 FF").to_vec(),
            ext: String::from("jpg"),
            carving_func: fourcc_carver::<JpegSegment, JpegSegment>,
            category: String::from("images/jpg"),
            min_size: min_size,
            max_size: 1000000,
            index: Mutex::new(0),
            carving_method: CarvingMethod::Strict,
        });

        Self(vec)
    }

    // list of patterns to give to the Aho-Corasick algorithm
    // basically, it's the list of magic bytes
    pub fn patterns(&self) -> anyhow::Result<AhoCorasick> {
        // Define binary patterns to search for
        let patterns: Vec<_> = self.0.iter().map(|ftype| ftype.magic.clone()).collect();

        // Build the Aho-Corasick automaton
        let ac = AhoCorasickBuilder::new().build(&patterns)?;

        Ok(ac)
    }

    // only keep the extensions found in the list passed
    pub fn retain(&mut self, ext_list: &[String]) {
        if !ext_list.is_empty() {
            self.0.retain(|ft| ext_list.contains(&ft.ext));
        }
    }
}

// impl Index<usize> for Corpus {
//     type Output = Option<String>;

//     fn index(&self, i: usize) -> &Self::Output {
//         self.0.iter().find(|)

//     }
// }

impl Deref for Corpus {
    type Target = Vec<FileType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
