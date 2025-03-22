// main ffunction for searching patterns in a segment
// a segment is made of a starting and ending offset

use std::collections::HashMap;

use crate::{carver::carve_using_size, filetypes::bmp::BMP, patterns::FileTypeCounter};

use aho_corasick::AhoCorasick;
use indicatif::ProgressBar;
use log::debug;

#[derive(Debug)]
pub struct Context<'a> {
    pub chunk: &'a [u8],                  // the bytes buffer to search
    pub buffer_size: usize,               // size of the window to search
    pub min_size: usize,                  // minium size of a file to be carved
    pub pb: &'a ProgressBar,              // ref on progress bar
    pub ac: &'a AhoCorasick,              // ref on Aho-Corasick engine
    pub ft: &'a mut FileTypeCounter, // ref on the file types counter
}
// #[derive(Debug)]
// pub struct Context<'a> {
//     pub mmap: &'a [u8],               // the bytes buffer to search
//     pub starting_offset: u64,         // the offset from which to search
//     pub ending_offset: u64,           // the offset up to which to search
//     pub buffer_size: usize,           // size of the window to search
//     pub min_size: usize,              // minium size of a file to be carved
//     pub pb: &'a ProgressBar,          // ref on progress bar
//     pub ac: &'a AhoCorasick,          // ref on Aho-Corasick engine
//     pub ft: &'a mut HashMap<String, u32>, // ref on the file types counter
// }

pub fn search(ctx: &mut Context) -> anyhow::Result<()> {
    // loop through bytes trying to discover some patterns
    let mut offset = 0u64;

    // while offset < ctx.chunk.len() as u64 {
    //     let buf = window(
    //         &ctx.mmap,
    //         ctx.ending_offset,
    //         ctx.starting_offset,
    //         ctx.buffer_size,
    //     );
    //     let mut hop = buf.len() as u64;

        for mat in ctx.ac.find_iter(&ctx.chunk) {
            debug!(
                "Found pattern {:?} at offset 0x{:X?}",
                mat.pattern().as_u64(),
                offset + mat.start() as u64
            );

            let found_offset = offset + mat.start() as u64;

            let x = carve_using_size::<BMP>(
                &ctx.chunk[found_offset as usize..],
                &mut ctx.ft,
                ctx.min_size,
                ctx.pb
            )?;

            if x == 0 {
                continue;
            }

            ctx.pb.set_position(x);

            // hop = x;
            // break;

            // // fs::write("output.bin", &buf)?; // Saves buffer to file
            // // std::process::exit(1);
        }

        // // move forward
        // offset += hop;

        // // update bar
        // ctx.pb.set_position(offset);
    // }

    Ok(())
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
