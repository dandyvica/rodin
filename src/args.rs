//! Manage command line arguments here.
use std::fs::OpenOptions;
use std::path::PathBuf;

use clap::builder::styling;
use clap::{Arg, ArgAction, Command};
use simplelog::*;

/// This structure holds the command line arguments.
#[derive(Debug, Default)]
pub struct CliOptions {
    // input file to analyze
    pub input_file: PathBuf,

    // buffer size used to look for patterns
    pub buffer_size: usize,

    // minimum file size to consider
    pub min_size: usize,

    // number of threads to use
    pub nb_threads: usize,

    // only carve those file types
    pub ext_list: Vec<String>,

    // display progress bar
    pub progress_bar: bool,

    // maximum number of files to carve, after that, stops
    pub limit: Option<usize>,
}

impl CliOptions {
    #[allow(clippy::field_reassign_with_default)]
    pub fn new() -> anyhow::Result<CliOptions> {
        const STYLES: styling::Styles = styling::Styles::styled()
            .header(styling::AnsiColor::Green.on_default().bold())
            .usage(styling::AnsiColor::Green.on_default().bold())
            .literal(styling::AnsiColor::Blue.on_default().bold())
            .placeholder(styling::AnsiColor::Cyan.on_default());

        let matches = Command::new("rodin, another file carver")
            .version("0.1")
            .author("Alain Viguier dandyvica@gmail.com")
            .styles(STYLES)
            .about(
                r#"Another file carver.

            Project home page: https://github.com/dandyvica/rodin
            
            "#,
            )
            .arg(
                Arg::new("input")
                    .short('i')
                    .long("input")
                    .long_help("Name and path of the input file to be carved")
                    .value_name("FILE")
                    .value_parser(clap::value_parser!(PathBuf))
                    .required(true),
            )
            .arg(
                Arg::new("buffer")
                    .short('b')
                    .long("buffer")
                    .long_help("Length in bytes of the buffer used to look for patterns")
                    .value_name("BUFFER")
                    .value_parser(clap::value_parser!(usize))
                    .default_missing_value("4096")
                    .required(false),
            )
            .arg(
                Arg::new("minsize")
                    .short('m')
                    .long("minsize")
                    .long_help("If discovered file length is less then SIZE, it'll be not carved")
                    .value_name("SIZE")
                    .value_parser(clap::value_parser!(usize))
                    .required(false),
            )
            .arg(
                Arg::new("nbthreads")
                    .short('n')
                    .long("nbthreads")
                    .long_help("Number of threads to use to split the carving")
                    .value_name("THREADS")
                    .value_parser(clap::value_parser!(usize))
                    .required(false),
            )
            .arg(
                Arg::new("limit")
                    .short('l')
                    .long("limit")
                    .long_help("Stop carving after <LIMIT> files")
                    .value_name("LIMIT")
                    .value_parser(clap::value_parser!(usize))
                    .required(false),
            )
            .arg(
                Arg::new("log")
                    .long("log")
                    .long_help("Save debugging info into the file LOG.")
                    .action(ArgAction::Set)
                    .value_name("LOG")
                    .value_parser(clap::value_parser!(PathBuf)),
            )
            .arg(
                Arg::new("verbose")
                    .short('v')
                    .long("verbose")
                    .long_help("Verbose mode, from info (-v) to trace (-vvvvv).")
                    .action(ArgAction::Count),
            )
            .arg(
                Arg::new("pb")
                    .long("progress")
                    .short('p')
                    .action(ArgAction::SetTrue)
                    .long_help("Display progress bar"),
            )
            .arg(
                Arg::new("ext")
                    .short('e')
                    .long("ext")
                    .help("Comma-separated list of extensions to carve")
                    .num_args(1)
                    .value_delimiter(',')
                    .required(false),
            )
            .get_matches();

        // save all cli options into a structure
        let mut options = CliOptions::default();

        // input file & layout file are mandatory. Try to canonicalize() at the same time.
        options.input_file = matches.get_one::<PathBuf>("input").unwrap().clone();
        options.buffer_size = *matches.get_one::<usize>("buffer").unwrap_or_else(|| &4096);
        options.min_size = *matches.get_one::<usize>("minsize").unwrap_or(&0);
        options.nb_threads = *matches.get_one::<usize>("nbthreads").unwrap_or(&1);
        options.limit = matches.get_one::<usize>("limit").copied();

        options.ext_list = matches
            .get_many::<String>("ext")
            .unwrap()
            .map(|s| s.clone())
            .collect();

        // manage debugging
        if matches.contains_id("verbose") {
            let level = match matches.get_count("verbose") {
                0 => log::LevelFilter::Off,
                1 => log::LevelFilter::Info,
                2 => log::LevelFilter::Warn,
                3 => log::LevelFilter::Error,
                4 => log::LevelFilter::Debug,
                5..=255 => log::LevelFilter::Trace,
            };
            if let Some(path) = matches.get_one::<PathBuf>("log") {
                init_write_logger(path, level)?;
            } else {
                init_term_logger(level)?;
            }
        }

        // set pb
        options.progress_bar = matches.get_flag("pb");

        Ok(options)
    }
}

// Initialize write logger: either create it or use it
fn init_write_logger(logfile: &PathBuf, level: log::LevelFilter) -> anyhow::Result<()> {
    if level == log::LevelFilter::Off {
        return Ok(());
    }

    // initialize logger
    let writable = OpenOptions::new().create(true).append(true).open(logfile)?;

    WriteLogger::init(
        level,
        simplelog::ConfigBuilder::new()
            .set_time_format_rfc3339()
            // .set_time_format_custom(format_description!(
            //     "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond]"
            .build(),
        writable,
    )?;

    Ok(())
}

// Initialize terminal logger
fn init_term_logger(level: log::LevelFilter) -> anyhow::Result<()> {
    if level == log::LevelFilter::Off {
        return Ok(());
    }
    TermLogger::init(
        level,
        Config::default(),
        TerminalMode::Stderr,
        ColorChoice::Auto,
    )?;

    Ok(())
}
