// all the carvers are located as modules here
use std::fmt::Debug;

// this is returned by the main seach function
#[derive(Debug, Default)]
pub struct CarvingResult {
    // if a file is found, we need to move the offset when saving that file
    pub offset: u64,

    // if a file is found, this is its file name
    pub file_name: Option<String>,

    // payload length is the artefact length
    pub length: usize,
/* 
    // sample bytes from offset
    pub sample: Vec<u8>, */
}

impl CarvingResult {
    // helper to define a new result
    pub fn new(offset: u64, file_name: &str, length: usize) -> Self {
        Self {
            offset,
            file_name: Some(String::from(file_name)),
            length
        }
    }
}

// carve when the file header contains the file size
pub mod fourcc_carver;
pub mod size_carver;
