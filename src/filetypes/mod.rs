use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
};

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use hex_literal::hex;
use wav::WAV;

use crate::{
    carvers::{CarvingResult, fourcc_carver::fourcc_carver, size_carver::carve_using_size},
    filetypes::bmp::BMP,
};

// alias for the carving function depending on the file type
pub type FileTypeCounter = Arc<Mutex<HashMap<String, u32>>>;
pub type CarvingFunc =
    fn(&[u8], &mut FileTypeCounter, &str, usize) -> anyhow::Result<CarvingResult>;

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
    pub min_size: u64,
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
        });

        // WAV
        vec.push(FileType {
            magic: b"RIFF".to_vec(),
            ext: String::from("wav"),
            carving_func: carve_using_size::<WAV>,
            category: String::from("audio/wav"),
            min_size: 10000,
        });

        // PNG
        vec.push(FileType {
            magic: hex!("89 50 4E 47 0D 0A 1A 0A").to_vec(),
            ext: String::from("png"),
            carving_func: fourcc_carver,
            category: String::from("images/png"),
            min_size: 10000,
        });

        Self(vec)
    }

    // create a file type counter to manage file creation with an index
    // this map is given to the carving function for it to be able to
    // define a name with an incrimental index (.e.g: bmp_00000002.bmp)
    // this is Arc<Mutex> because it'll be used and updated from different
    // threads
    pub fn index_map(&self) -> Arc<Mutex<HashMap<String, u32>>> {
        let mut map = HashMap::<String, u32>::new();

        // just init counter to 0
        for ftype in &self.0 {
            map.insert(ftype.ext.clone(), 0);
        }

        Arc::new(Mutex::new(map))
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

pub mod bmp;
pub mod wav;
