use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use hex_literal::hex;

pub fn patterns() -> AhoCorasick {
    // Define binary patterns to search for
    let patterns = vec![
        // vec![0x50, 0x4B, 0x03, 0x04], // ZIP file signature (PK)
        // hex!("89 50 4E 47 0D 0A 1A 0A").to_vec(), // PNG file signature
        b"BM".to_vec(), // BMP file signature
    ];

    // Convert patterns to &[u8] slices
    let patterns_refs: Vec<&[u8]> = patterns.iter().map(|p| p.as_slice()).collect();

    // Build the Aho-Corasick automaton
    let ac = AhoCorasickBuilder::new().build(&patterns_refs).unwrap();

    ac
}

pub type FileTypeCounter = Arc<Mutex<HashMap<String, u32>>>;
pub fn filetype_counter() -> Arc<Mutex<HashMap<String, u32>>> {
    let mut h = HashMap::<String, u32>::new();

    h.insert(String::from("bmp"), 0);
    h.insert(String::from("wav"), 0);

    Arc::new(Mutex::new(h))
}
