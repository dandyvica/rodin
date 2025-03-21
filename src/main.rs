use std::fs::{self, File};

use indicatif::{ProgressBar, ProgressStyle};
use log::debug;
use memmap2::MmapOptions;

mod patterns;
use patterns::{filetype_index, patterns};

mod args;
use args::CliOptions;

mod carver;
use carver::carve_using_size;

mod filetypes;
use filetypes::bmp::BMP;

fn main() -> anyhow::Result<()> {
    // harvest cli arguments
    let opts = CliOptions::new()?;

    // open image and build mmap
    let file = File::open(&opts.input_file)?;
    let metadata = file.metadata()?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };

    let size = metadata.len();
    let mut offset = 0u64;

    let mut ft = filetype_index();

    // build aho-corasick engine
    let ac = patterns();

    // buld the progress bar
    let pb = ProgressBar::new(size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
            .unwrap(),
    );

    let buffer_size = opts.buffer_size;

    // loop through bytes trying to discover some patterns
    while offset < size {
        let buf = window(&mmap, size, offset, buffer_size);
        let mut hop = buf.len() as u64;

        for mat in ac.find_iter(&buf) {
            debug!(
                "Found pattern {:?} at offset 0x{:X?}",
                mat.pattern().as_u64(),
                offset + mat.start() as u64
            );

            let found_offset = offset + mat.start() as u64;

            let x =
                carve_using_size::<BMP>(&mmap[found_offset as usize..], &mut ft, opts.min_size)?;

            if x == 0 {
                continue;
            }

            hop = x;
            break;

            // fs::write("output.bin", &buf)?; // Saves buffer to file
            // std::process::exit(1);
        }

        // move forward
        offset += hop;

        // update bar
        pb.set_position(offset);
    }

    Ok(())
}

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
