use std::{
    collections::HashMap, fs::{self, File}, ops::Mul, sync::{Arc, Mutex}, thread
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::debug;
use memmap2::MmapOptions;

mod patterns;
use patterns::{filetype_counter, patterns};

mod args;
use args::CliOptions;

mod carver;
use carver::carve_using_size;

mod search;
use search::{Context, search};

mod filetypes;
use filetypes::bmp::BMP;

fn main() -> anyhow::Result<()> {
    // harvest cli arguments
    let opts = CliOptions::new()?;

    // open image and build mmap
    let file = File::open(&opts.input_file)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let mmap = Arc::new(mmap);

    let ftype_counter = filetype_counter();

    // build aho-corasick engine
    // let ac = patterns();

    // create a MultiProgress object to manage multiple progress bars
    let multi_progress = Arc::new(MultiProgress::new());

    // compute the different segments according to the number of threads
    let mut handles = vec![];
    let chunk_size = mmap.len() / opts.nb_threads;

    for i in 0..opts.nb_threads {
        let mmap_clone = Arc::clone(&mmap); // Clone Arc for each thread
        let multi_progress_clone = Arc::clone(&multi_progress);
        let mut ftype_counter_clone = Arc::clone(&ftype_counter);

        // let mut map = ftype_counter_clone.lock().unwrap();
        // map.insert(String::from("key"), 42);

        // spawn thread
        let handle = thread::spawn(move || {

            println!("================== starting thread {}",  i);

            // calculate the buffer offsets
            let start = i * chunk_size;
            let end = if i == opts.nb_threads - 1 {
                mmap_clone.len()
            } else {
                start + chunk_size
            };

            // define a progress bar
            let pb = multi_pbar(&multi_progress_clone, end, i);

            // define an ew Aho-Corasick engine
            let ac = patterns();

            // each thread processes its assigned chunk
            let chunk = &mmap_clone[start..end];

            // now search within each chunk
            let mut ctx = Context {
                chunk: chunk,
                buffer_size: opts.buffer_size,
                min_size: opts.min_size,
                pb: &pb,
                ac: &ac,
                ft: &mut ftype_counter_clone,
            };

            search(&mut ctx);

            println!("????????? {}", i);
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut i = 0;
    for handle in handles {
        match handle.join() {
            Ok(_) => println!("Thread {} finished successfully!", i),
            Err(_) => println!("Thread {} panicked!", i),
        }
        i += 1;
    }

    Ok(())
}

// define mutli-progress bars, one for each thread
fn multi_pbar(mp: &Arc<MultiProgress>, length: usize, thread_number: usize) -> ProgressBar {
    let pb = mp.add(ProgressBar::new(length as u64));

    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{msg}] {bar:40.cyan/blue} {pos}/{len}")
            .unwrap(),
    );
    // pb.set_style(
    //     ProgressStyle::default_bar()
    //         .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
    //         .unwrap(),
    // );
    // pb.set_message(format!("Thread {}", thread_number + 1));

    pb
}
