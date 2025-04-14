use std::{
    fs::File,
    ops::Range,
    sync::{Arc, atomic::AtomicUsize},
    thread,
    time::Instant,
};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, info, trace};
use memmap2::MmapOptions;

mod args;
use args::CliOptions;

mod carvers;

mod search;
use search::Context;

mod filetypes;
use filetypes::corpus::Corpus;

mod deserializer;

fn main() -> anyhow::Result<()> {
    // harvest cli arguments
    let opts = CliOptions::new()?;
    trace!("args: {:?}", opts);
    let now = Instant::now();

    // open image and build mmap
    let file = File::open(&opts.input_file)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let mmap = Arc::new(mmap);

    // build our patterns and optionally retain only file types that are passed in the cli
    let mut corpus = Corpus::new(opts.min_size);
    corpus.retain(&opts.ext_list);

    let corpus = Arc::new(corpus);

    // counter on the number of files currently carved out, for all threads
    let nb_files = Arc::new(AtomicUsize::new(0));

    // build patterns and aho-corasick engine
    let ac = Arc::new(corpus.patterns()?);

    // create a MultiProgress object to manage multiple progress bars
    let multi_progress = Arc::new(MultiProgress::new());

    // compute the different chunk according to the number of threads
    // mmap is divided in several nb_threads chunks, each chunk is provided to a thread
    let mut handles = vec![];
    let chunk_size = mmap.len() / opts.nb_threads;

    for i in 0..opts.nb_threads {
        // clone what is needed
        let mmap_clone = Arc::clone(&mmap); // Clone Arc for each thread
        let multi_progress_clone = Arc::clone(&multi_progress);
        let ac_clone = Arc::clone(&ac);
        let corpus_clone = Arc::clone(&corpus);
        let nb_files_clone = Arc::clone(&nb_files);

        // spawn thread
        let handle = thread::spawn(move || -> anyhow::Result<usize> {
            info!("starting thread {}", i);

            // calculate the buffer offsets
            let start = i * chunk_size;
            let end = if i == opts.nb_threads - 1 {
                mmap_clone.len()
            } else {
                start + chunk_size
            };

            // we pass the range to the search function
            let rg = Range { start, end };

            // define a progress bar dedicated to this thread
            let pb = multi_pbar(&multi_progress_clone, end - start, i);
            pb.set_message("Searching..................");

            // now search within each chunk
            let mut ctx = Context {
                mmap: &mmap_clone,
                bounds: rg,
                pb: &pb,
                ac: &ac_clone,
                corpus: &corpus_clone,
                nb_files: &nb_files_clone,
            };

            let found = ctx.search(&opts.limit)?;

            // end of thread
            pb.set_message(format!("thread finished, {} files found", found));
            pb.finish();

            Ok(found)
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut thread_id = 0;
    let mut total_count = 0usize;

    for handle in handles {
        match handle.join() {
            Ok(res) => match res {
                Ok(count) => total_count += count,
                Err(e) => info!("Thread {} finished successfully!", thread_id),
            },
            Err(_) => info!("Thread {} panicked!", thread_id),
        }
        thread_id += 1;
    }

    // print out statistics
    let elapsed = now.elapsed();
    println!("total time: {:?}", elapsed);

    Ok(())
}

// define mutli-progress bars, one for each thread
fn multi_pbar(mp: &Arc<MultiProgress>, length: usize, thread_number: usize) -> ProgressBar {
    let pb = mp.add(ProgressBar::new(length as u64));

    pb.set_style(
        ProgressStyle::default_bar()
            .template("[{msg}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
            .unwrap(),
    );

    pb.set_message(format!("Thread {}", thread_number + 1));

    // pb.set_style(
    //     ProgressStyle::default_bar()
    //         .template("[{elapsed_precise}] {bar:40.cyan/blue} {bytes}/{total_bytes} ({eta})")
    //         .unwrap(),
    // );
    // pb.set_message(format!("Thread {}", thread_number + 1));

    pb
}
