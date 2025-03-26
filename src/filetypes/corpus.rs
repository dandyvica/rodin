use std::{
    fs::{self, File},
    io::{BufWriter, Write},
    ops::Deref,
    path::Path,
    sync::Mutex,
};

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use hex_literal::hex;

use crate::{
    carvers::{CarvingResult, fourcc_carver::fourcc_carver, size_carver::carve_using_size},
    filetypes::{bmp::BMP, wav::WAV},
};

use super::png::{PNGChunk, PNGHeader};

// alias for the carving function depending on the file type
pub type CarvingFunc = fn(&[u8], &FileType) -> anyhow::Result<CarvingResult>;

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

    // the current index of the file being carved
    pub index: Mutex<usize>,
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
    pub fn new() -> Self {
        let mut vec = Vec::new();

        // BMP
        vec.push(FileType {
            magic: b"BM".to_vec(),
            ext: String::from("bmp"),
            carving_func: carve_using_size::<BMP>,
            category: String::from("images/bmp"),
            min_size: 10000,
            index: Mutex::new(0),
        });

        // WAV
        vec.push(FileType {
            magic: b"RIFF".to_vec(),
            ext: String::from("wav"),
            carving_func: carve_using_size::<WAV>,
            category: String::from("audio/wav"),
            min_size: 10000,
            index: Mutex::new(0),
        });

        // PNG
        vec.push(FileType {
            magic: hex!("89 50 4E 47 0D 0A 1A 0A").to_vec(),
            ext: String::from("png"),
            carving_func: fourcc_carver::<PNGHeader, PNGChunk>,
            category: String::from("images/png"),
            min_size: 10000,
            index: Mutex::new(0),
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
}

impl Deref for Corpus {
    type Target = Vec<FileType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
