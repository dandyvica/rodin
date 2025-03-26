use std::{fs::File, ops::Range, sync::Arc, thread};

use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use log::{debug, info};
use memmap2::MmapOptions;

mod args;
use args::CliOptions;

mod carvers;

mod search;
use search::{Context, search};

mod filetypes;
use filetypes::corpus::Corpus;

mod deserializer;

fn main() -> anyhow::Result<()> {
    // harvest cli arguments
    let opts = CliOptions::new()?;

    // open image and build mmap
    let file = File::open(&opts.input_file)?;
    let mmap = unsafe { MmapOptions::new().map(&file)? };
    let mmap = Arc::new(mmap);

    // build our patterns
    let corpus = Arc::new(Corpus::new());

    // build patterns and aho-corasick engine
    let ac = Arc::new(corpus.patterns()?);

    // create a MultiProgress object to manage multiple progress bars
    let multi_progress = Arc::new(MultiProgress::new());

    // compute the different chunk according to the number of threads
    // mmap is divided in several nb threads chunks, each chunk is provided to a thread
    // bu

    let mut handles = vec![];
    let chunk_size = mmap.len() / opts.nb_threads;

    for i in 0..opts.nb_threads {
        // clone what is needed
        let mmap_clone = Arc::clone(&mmap); // Clone Arc for each thread
        let multi_progress_clone = Arc::clone(&multi_progress);
        let ac_clone = Arc::clone(&ac);
        let corpus_clone = Arc::clone(&corpus);

        // spawn thread
        let handle = thread::spawn(move || {
            info!("================== starting thread {}", i);

            // calculate the buffer offsets
            let start = i * chunk_size;
            let end = if i == opts.nb_threads - 1 {
                mmap_clone.len()
            } else {
                start + chunk_size
            };

            // we pass the range to the search function
            let rg = Range { start, end };

            // define a progress bar
            let pb = multi_pbar(&multi_progress_clone, end - start, i);
            pb.set_message("Searching.......");

            // each thread processes its assigned chunk
            // let chunk = &mmap_clone[start..end];

            // now search within each chunk
            let mut ctx = Context {
                mmap: &mmap_clone,
                bounds: rg,
                pb: &pb,
                ac: &ac_clone,
                corpus: &corpus_clone,
            };

            let found = search(&mut ctx).unwrap();

            // end of thread
            pb.set_message(format!("thread finished, {} files found", found));
            pb.finish();
        });

        handles.push(handle);
    }

    // Wait for all threads to complete
    let mut i = 0;
    for handle in handles {
        match handle.join() {
            Ok(_) => info!("Thread {} finished successfully!", i),
            Err(_) => info!("Thread {} panicked!", i),
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
