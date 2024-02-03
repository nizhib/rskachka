mod abort;
mod args;
mod fetch;
mod images;
mod item;
mod saving;
mod worker;

use std::{
    fmt::Write,
    fs::File,
    io::{BufRead, BufReader, Result},
    process,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use clap::Parser;
use clap_verbosity_flag::{LogLevel, Verbosity};
use crossbeam::channel::{bounded, Receiver, Sender};
use indicatif::{ProgressBar, ProgressFinish, ProgressState, ProgressStyle};
use log::{info, warn, Level};

use crate::abort::break_on_flag;
use crate::args::Args;
use crate::saving::SavingSemaphore;
use crate::worker::ProcessOptions;

const BAR_TEMPLATE: &str =
    "{percent:>3}% |{wide_bar}| {human_pos}/{human_len} [{elapsed}<{eta}, {my_per_sec}]";

fn parse_args() -> Result<Args> {
    let args = Args::parse();
    if args.progress && args.verbose.log_level().unwrap_or(Level::Error) > Level::Warn {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Choose either verbose logging or progress display, not both",
        ))
    } else {
        Ok(args)
    }
}

fn init_logging<T: LogLevel>(verbose: &Verbosity<T>) {
    env_logger::Builder::new()
        .filter_level(verbose.log_level_filter())
        .init();
}

fn open_index_file(file_path: &str) -> Result<File> {
    File::open(file_path).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Index file not found: {}", file_path),
        )
    })
}

fn calculate_index_size(file_path: &str, no_header: bool) -> Result<usize> {
    let index_file = open_index_file(file_path)?;

    info!("Calculating the index size...");
    let mut index_lines = BufReader::new(index_file).lines();
    let Ok(mut index_size) = index_lines.try_fold(0, |acc, line| line.map(|_| acc + 1)) else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Error calculating the index size",
        ));
    };
    if !no_header {
        index_size -= 1;
    }
    info!("Total entries found: {}", index_size);
    Ok(index_size)
}

fn create_progressbar(progress: bool, index_size: usize) -> Option<ProgressBar> {
    if progress {
        Some(
            ProgressBar::new(index_size as u64)
                .with_style(
                    ProgressStyle::with_template(BAR_TEMPLATE)
                        .unwrap()
                        .progress_chars("##-")
                        .with_key("my_per_sec", |s: &ProgressState, w: &mut dyn Write| {
                            write!(w, "{:.02}it/s", s.per_sec()).unwrap()
                        }),
                )
                .with_finish(ProgressFinish::AndLeave),
        )
    } else {
        None
    }
}

fn set_ctrl_c_handler(stopped: &Arc<AtomicBool>, saving: &Arc<SavingSemaphore>) {
    let c_stopped = Arc::clone(stopped);
    let c_saving = Arc::clone(saving);
    ctrlc::set_handler(move || {
        warn!("Waiting for the workers to shut down...");
        c_stopped.store(true, Ordering::Relaxed);
        c_saving.wait();
        warn!("Done!");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");
}

fn launch_producer(
    index_file: File,
    no_header: bool,
    work_tx: Sender<csv::StringRecord>,
    stopped: &Arc<AtomicBool>,
    pb: &Option<ProgressBar>,
) {
    let c_pb = pb.clone();
    let c_stopped = Arc::clone(stopped);
    thread::Builder::new()
        .name("producer".to_string())
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            for record in csv::ReaderBuilder::new()
                .has_headers(!no_header)
                .from_reader(index_file)
                .records()
            {
                break_on_flag!(c_stopped, || warn!("Shutting down the producer..."));
                match record {
                    Ok(record) => {
                        work_tx.send(record).unwrap();
                    }
                    Err(e) => {
                        if let Some(c_pb) = &c_pb {
                            c_pb.inc(1);
                        }
                        warn!("Error reading record: {}", e);
                    }
                };
            }
        })
        .unwrap();
}

fn launch_workers(
    worker_count: usize,
    work_rx: &Receiver<csv::StringRecord>,
    options: &ProcessOptions,
    stopped: &Arc<AtomicBool>,
    saving: &Arc<SavingSemaphore>,
    pb: &Option<ProgressBar>,
) {
    thread::scope(|s| {
        for i in 0..worker_count {
            thread::Builder::new()
                .name(format!("worker{}", i))
                .stack_size(4 * 1024 * 1024)
                .spawn_scoped(s, move || {
                    while let Ok(record) = work_rx.recv() {
                        if let Err(err) = worker::process(&record, options, stopped, saving) {
                            warn!("{}", err);
                        }
                        if let Some(pb) = &pb {
                            pb.inc(1);
                        }
                    }
                })
                .unwrap();
        }
    });
}

fn main() -> Result<()> {
    // Get the arguments
    let args = parse_args()?;

    // Set the log level
    init_logging(&args.verbose);

    // Calculate the index size
    let index_size = calculate_index_size(&args.index_path, args.no_header)?;

    // Reopen the index file
    let index_file = open_index_file(&args.index_path)?;

    // Set up the communication
    let (work_tx, work_rx) = bounded::<csv::StringRecord>(args.worker_count);
    let stopped = Arc::new(AtomicBool::new(false));
    let saving = Arc::new(SavingSemaphore::new());

    // Create a progressbar
    let pb = create_progressbar(args.progress, index_size);

    // Gracefully shutdown on Ctrl-C
    set_ctrl_c_handler(&stopped, &saving);

    // Launch the producer
    launch_producer(index_file, args.no_header, work_tx, &stopped, &pb);

    // Launch the workers
    launch_workers(
        args.worker_count,
        &work_rx,
        &args.into(),
        &stopped,
        &saving,
        &pb,
    );

    Ok(())
}
