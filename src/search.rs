// main ffunction for searching patterns in a segment
// a segment is made of a starting and ending offset

use std::{
    ops::Range,
    sync::{
        Mutex,
        atomic::{AtomicUsize, Ordering},
    },
};

use crate::{
    audit::{AuditData, AuditFile},
    filetypes::corpus::Corpus,
};

use aho_corasick::AhoCorasick;
use indicatif::ProgressBar;
use log::{debug, info, trace};

#[derive(Debug)]
pub struct Context<'a> {
    pub mmap: &'a [u8],                   // the mmap to search
    pub bounds: Range<usize>,             // contains the bounds of the chunk to search
    pub pb: &'a ProgressBar,              // ref on progress bar
    pub ac: &'a AhoCorasick,              // ref on Aho-Corasick engine
    pub corpus: &'a Corpus,               // ref on global corpus
    pub nb_files: &'a AtomicUsize,        // ref on the global number of file currently carved out
    pub audit_file: &'a Mutex<AuditFile>, // ref on audit file
}

impl<'a> Context<'a> {
    // add 1 to the global number of file currently carved out
    /*     pub fn one_more(&mut self) {
        self.nb_files.fetch_add(1, Ordering::Relaxed);
    } */

    // try to carve file using the context. This means trying out all patterns using the Aho-Corasick
    // algorithm to get potential file signatures, and call the carving function to try to carve
    pub fn search(&mut self, limit: &Option<usize>) -> anyhow::Result<usize> {
        trace!("bounds={:?}", self.bounds);

        // loop through bytes trying to discover some patterns
        let absolute_offset = self.bounds.start;

        // we count the number of files found for each thread
        let mut files_found = 0usize;

        // we're searching patterns inside this chunk/segment
        let chunk = &self.mmap[self.bounds.clone()];

        // loop through the pattern we found
        // a found pattern doesn't mean it's a genuine file. It's a potentialty
        for mat in self.ac.find_iter(chunk) {
            let pat_index = mat.pattern().as_usize();
            let pat = &self.corpus.get(pat_index).unwrap().ext;
            debug!(
                "Found pattern '{}' at offset 0x{:X?}({})",
                pat,
                mat.start(),
                mat.start()
            );

            // pattern returned contains the index of the pattern inside the corpus
            let ft = self
                .corpus
                .get(mat.pattern().as_usize())
                .expect("error getting magic");

            // the carving func is the function used to try to carve for a specific file type
            let carving_func = ft.carving_func;

            // absolute_found_offset is the pattern offset found for the while file
            let absolute_found_offset = absolute_offset + mat.start();

            // mat.start() is the pattern offset found for the chunk
            self.pb.set_position(mat.start() as u64);

            // let ft = Arc::new(ft);
            // println!("starting carving at offset: {}", absolute_found_offset);
            let result = carving_func(&self.mmap[absolute_found_offset..], ft)?;

            // offset returned is 0, we didn't find/carve any artefact
            if result.offset == 0 {
                continue;
            }

            // update progress bar with the file name being carved
            let file_name = result.file_name.unwrap();
            info!(
                "found and carved artefact ({}) at offsets: 0x{:X?}-0x{:X?}",
                file_name,
                absolute_found_offset,
                absolute_found_offset as u64 + result.offset
            );

            // save audit data
            let ad = AuditData {
                artefact: &file_name.as_str(),
                offset_start: absolute_found_offset as u64,
                offset_end: absolute_found_offset as u64 + result.offset,
                length: result.length as u64,
            };
            {
                if let Ok(mut ul) = self.audit_file.try_lock() {
                    ul.add_artefact(&ad)?;
                }
            }

            // print out file name on progress bar
            self.pb.set_message(file_name);

            // stop carving is we reached the limit
            files_found += 1;
            if let Some(limit) = limit {
                if files_found > *limit {
                    return Ok(files_found);
                }
            }
        }

        Ok(files_found)
    }
}

// fn window(mmap: &[u8], size: u64, offset: u64, buffer_size: usize) -> &[u8] {
//     let lower = offset as usize;
//     let upper = if lower + buffer_size < (size as usize) {
//         lower + buffer_size
//     } else {
//         size as usize
//     };

//     &mmap[lower..upper]
// }

// #[cfg(test)]
// mod tests {

//     use super::*;

//     fn window_() {
//         let buf = b"AAAAAAAAAABBBBBBBBBB";

//         let w = window(&buf.as_slice(), buf.len() as u64, 0, 5);
//         assert_eq!(w.len(), 5);
//         assert_eq!(w, b"AAAAA");

//         let w = window(&buf.as_slice(), buf.len() as u64, 5, 10);
//         assert_eq!(w.len(), 10);
//         assert_eq!(w, b"AAAAABBBBB");

//         let w = window(&buf.as_slice(), buf.len() as u64, 10, 10);
//         assert_eq!(w.len(), 10);
//         assert_eq!(w, b"BBBBBBBBBB");

//         let w = window(&buf.as_slice(), buf.len() as u64, 15, 10);
//         assert_eq!(w.len(), 5);
//         assert_eq!(w, b"BBBBB");

//         let w = window(&buf.as_slice(), buf.len() as u64, 19, 10);
//         assert_eq!(w.len(), 5);
//         assert_eq!(w, b"BB");
//     }
// }
