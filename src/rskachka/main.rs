mod abort;
mod args;
mod fetcher;
mod images;
mod saving;
mod worker;

use std::{
    fs::File,
    io::Result,
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
use indicatif::ProgressBar;
use log::{warn, Level};
use memmap2::Mmap;
use rskachka::{maybe_create_progressbar, rslc};

use crate::abort::break_on_flag;
use crate::args::Args;
use crate::saving::SavingSemaphore;
use crate::worker::Worker;

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

fn open_source_file(file_path: &str) -> Result<File> {
    File::open(file_path).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Index file not found: {}", file_path),
        )
    })
}

fn calculate_source_size(file_path: &str, no_header: bool) -> Result<usize> {
    let source_file = File::open(file_path)?;
    let source_mmap = unsafe { Mmap::map(&source_file) }?;
    Ok(rslc::count_lines(&source_mmap) - if no_header { 0 } else { 1 })
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
    source_file: File,
    no_header: bool,
    work_tx: Sender<csv::StringRecord>,
    stopped: &Arc<AtomicBool>,
    pb: &Option<ProgressBar>,
) {
    let c_stopped = Arc::clone(stopped);
    let c_pb = pb.clone();
    thread::Builder::new()
        .name("producer".to_string())
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            for record in csv::ReaderBuilder::new()
                .has_headers(!no_header)
                .from_reader(source_file)
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
    args: &Args,
    work_rx: &Receiver<csv::StringRecord>,
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
                    let worker = Worker::from(args);
                    while let Ok(record) = work_rx.recv() {
                        if let Err(err) = worker.process(&record, stopped, saving) {
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

    // Calculate the source size
    let source_size = calculate_source_size(&args.source_path, args.no_header)?;

    // Reopen the source file
    let source_file = open_source_file(&args.source_path)?;

    // Set up the communication
    let (work_tx, work_rx) = bounded::<csv::StringRecord>(args.worker_count);
    let stopped = Arc::new(AtomicBool::new(false));
    let saving = Arc::new(SavingSemaphore::new());

    // Create a progressbar
    let pb = maybe_create_progressbar(args.progress, source_size as u64);

    // Gracefully shutdown on Ctrl-C
    set_ctrl_c_handler(&stopped, &saving);

    // Launch the producer
    launch_producer(source_file, args.no_header, work_tx, &stopped, &pb);

    // Launch the workers
    launch_workers(args.worker_count, &args, &work_rx, &stopped, &saving, &pb);

    Ok(())
}
