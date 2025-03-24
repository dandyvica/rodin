// main ffunction for searching patterns in a segment
// a segment is made of a starting and ending offset

use std::{collections::HashMap, ops::Range, sync::Arc};

use crate::{
    carvers::size_carver::carve_using_size,
    filetypes::{Corpus, FileTypeCounter, bmp::BMP},
};

use aho_corasick::AhoCorasick;
use indicatif::ProgressBar;
use log::debug;
use memmap2::Mmap;

#[derive(Debug)]
pub struct Context<'a> {
    pub mmap: &'a [u8],              // the mmap to search
    pub bounds: Range<usize>,        // contains the bounds of the chunk to search
    pub buffer_size: usize,          // size of the window to search
    pub min_size: usize,             // minium size of a file to be carved
    pub pb: &'a ProgressBar,         // ref on progress bar
    pub ac: &'a AhoCorasick,         // ref on Aho-Corasick engine
    pub ft: &'a mut FileTypeCounter, // ref on the file types counter
    pub corpus: &'a Corpus,          // ref on global corpus
}

pub fn search(ctx: &mut Context) -> anyhow::Result<usize> {
    // loop through bytes trying to discover some patterns
    let mut offset = 0u64;
    let absolute_offset = ctx.bounds.start;

    // we count the number of files found for each thread
    let mut files_found = 0usize;

    // we're searching to this chunk
    let chunk = &ctx.mmap[ctx.bounds.clone()];

    // loop through what we found
    for mat in ctx.ac.find_iter(&chunk) {
        debug!(
            "Found pattern {:?} at offset 0x{:X?}",
            mat.pattern().as_u64(),
            mat.start() as u64
        );

        // pattern returned contains the index of the pattern
        let ft = ctx
            .corpus
            .get(mat.pattern().as_usize())
            .expect("error getting magic");
        let carving_func = ft.carving_func;

        let absolute_found_offset = absolute_offset + mat.start();
        ctx.pb.set_position(mat.start() as u64);

        let res = carving_func(
            &ctx.mmap[absolute_found_offset..],
            &mut ctx.ft,
            &ft.category,
            ctx.min_size,
        )?;

        // offset returned is 0, the so called file is not carved
        if res.offset == 0 {
            continue;
        }

        // update progress bar with file name being carved
        let file_name = res.file_name.unwrap();
        debug!(
            "found image {} at offset {}",
            file_name, absolute_found_offset
        );
        ctx.pb.set_message(file_name);

        files_found += 1;
    }

    Ok(files_found)
}
// pub fn search(ctx: &mut Context) -> anyhow::Result<()> {
//     // loop through bytes trying to discover some patterns
//     while ctx.starting_offset < ctx.ending_offset {
//         let buf = window(
//             &ctx.mmap,
//             ctx.ending_offset,
//             ctx.starting_offset,
//             ctx.buffer_size,
//         );
//         let mut hop = buf.len() as u64;

//         for mat in ctx.ac.find_iter(&buf) {
//             debug!(
//                 "Found pattern {:?} at offset 0x{:X?}",
//                 mat.pattern().as_u64(),
//                 ctx.starting_offset + mat.start() as u64
//             );

//             let found_offset = ctx.starting_offset + mat.start() as u64;

//             let x = carve_using_size::<BMP>(
//                 &ctx.mmap[found_offset as usize..],
//                 &mut ctx.ft,
//                 ctx.min_size,
//             )?;

//             if x == 0 {
//                 continue;
//             }

//             hop = x;
//             break;

//             // fs::write("output.bin", &buf)?; // Saves buffer to file
//             // std::process::exit(1);
//         }

//         // move forward
//         ctx.starting_offset += hop;

//         // update bar
//         ctx.pb.set_position(ctx.starting_offset);
//     }

//     Ok(())
// }

fn window(mmap: &[u8], size: u64, offset: u64, buffer_size: usize) -> &[u8] {
    let lower = offset as usize;
    let upper = if lower + buffer_size < (size as usize) {
        lower + buffer_size
    } else {
        size as usize
    };

    &mmap[lower..upper]
}

#[cfg(test)]
mod tests {

    use super::*;

    fn window_() {
        let buf = b"AAAAAAAAAABBBBBBBBBB";

        let w = window(&buf.as_slice(), buf.len() as u64, 0, 5);
        assert_eq!(w.len(), 5);
        assert_eq!(w, b"AAAAA");

        let w = window(&buf.as_slice(), buf.len() as u64, 5, 10);
        assert_eq!(w.len(), 10);
        assert_eq!(w, b"AAAAABBBBB");

        let w = window(&buf.as_slice(), buf.len() as u64, 10, 10);
        assert_eq!(w.len(), 10);
        assert_eq!(w, b"BBBBBBBBBB");

        let w = window(&buf.as_slice(), buf.len() as u64, 15, 10);
        assert_eq!(w.len(), 5);
        assert_eq!(w, b"BBBBB");

        let w = window(&buf.as_slice(), buf.len() as u64, 19, 10);
        assert_eq!(w.len(), 5);
        assert_eq!(w, b"BB");
    }
}
