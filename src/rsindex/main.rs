mod args;

use std::{fs::File, thread};

use clap::Parser;
use crossbeam::channel::{bounded, Receiver, Sender};
use indicatif::ProgressBar;
use log::{error, warn};
use memmap2::Mmap;
use rskachka::{item::Item, maybe_create_progressbar, rslc};

use crate::args::Args;

type InputRecord = csv::StringRecord;

struct OutputRecord {
    record: csv::StringRecord,
    found: bool,
}

impl OutputRecord {
    fn iter(&self) -> impl Iterator<Item = &str> {
        self.record.iter()
    }
}

fn launch_producer(
    source_reader: csv::Reader<File>,
    work_tx: Sender<InputRecord>,
    pb: &Option<ProgressBar>,
) {
    let pb = pb.clone();
    thread::Builder::new()
        .name("producer".to_string())
        .spawn(move || {
            let mut source_reader = source_reader;

            for record in source_reader.records() {
                if let Ok(record) = record {
                    if let Err(e) = work_tx.send(record) {
                        warn!("Error sending record: {}", e);
                    }
                }
                if let Some(pb) = &pb {
                    pb.inc(1);
                }
            }
        })
        .unwrap();
}

fn launch_saver(
    index_writer: csv::Writer<File>,
    missing_writer: Option<csv::Writer<File>>,
    save_rx: Receiver<OutputRecord>,
) {
    thread::Builder::new()
        .name("saver".to_string())
        .spawn(move || {
            let mut index_writer = index_writer;
            let mut missing_writer = missing_writer;

            for record in save_rx {
                if record.found {
                    if let Err(e) = index_writer.write_record(record.iter()) {
                        warn!("Error adding index record: {}", e);
                    }
                } else if let Some(missing_writer) = &mut missing_writer {
                    if let Err(e) = missing_writer.write_record(record.iter()) {
                        warn!("Error adding missing record: {}", e);
                    }
                }
            }
        })
        .unwrap();
}

fn launch_workers(
    url_field: i8,
    output_root: &str,
    extension: &str,
    work_rx: &Receiver<InputRecord>,
    save_tx: &Sender<OutputRecord>,
) {
    thread::scope(|s| {
        for i in 0..work_rx.capacity().unwrap_or(1) {
            thread::Builder::new()
                .name(format!("worker{}", i))
                .spawn_scoped(s, move || {
                    while let Ok(mut record) = work_rx.recv() {
                        match Item::from_record(&record, &[0], url_field, output_root, extension) {
                            Ok(item) => {
                                if item.path.exists() {
                                    record.extend([item.path.to_str().unwrap()]);
                                    save_tx
                                        .send(OutputRecord {
                                            record,
                                            found: true,
                                        })
                                        .unwrap();
                                } else {
                                    save_tx
                                        .send(OutputRecord {
                                            record,
                                            found: false,
                                        })
                                        .unwrap();
                                }
                            }
                            Err(e) => {
                                warn!("Error parsing record: {}", e);
                            }
                        }
                    }
                })
                .unwrap();
        }
    });
}

pub fn main() -> std::io::Result<()> {
    let args = Args::parse();

    // Open the source file and count lines
    let source_file = match File::open(&args.source_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Error opening source file: {}", e);
            return Ok(());
        }
    };
    let source_mmap = match unsafe { Mmap::map(&source_file) } {
        Ok(mmap) => mmap,
        Err(e) => {
            error!("Error mmaping source file: {}", e);
            return Ok(());
        }
    };
    let source_lines = rslc::count_lines(&source_mmap);

    // Open the source file again and create a reader
    let mut source_reader = csv::ReaderBuilder::new()
        .has_headers(!args.no_header)
        .from_path(args.source_path)?;

    let header = if args.no_header {
        None
    } else {
        Some(source_reader.headers()?)
    };

    // Open the index file and create a writer
    let index_file = match File::create(&args.index_path) {
        Ok(file) => file,
        Err(e) => {
            error!("Error creating index file: {}", e);
            return Ok(());
        }
    };
    let mut index_writer = csv::Writer::from_writer(index_file);

    // Maybe open the missing file and create a writer
    let mut missing_writer = if args.missing_path.is_empty() {
        None
    } else {
        let missing_file = match File::create(&args.missing_path) {
            Ok(file) => file,
            Err(e) => {
                error!("Error creating missing file: {}", e);
                return Ok(());
            }
        };
        Some(csv::Writer::from_writer(missing_file))
    };

    // Create a progressbar
    let pb = maybe_create_progressbar(args.progress, source_lines as u64);

    // Write the headers if any
    if let Some(header) = header {
        if missing_writer.is_some() {
            if let Err(e) = missing_writer.as_mut().unwrap().write_record(header.iter()) {
                error!("Error adding missing header: {}", e);
                return Ok(());
            }
        }
        let mut header = header.to_owned();
        header.extend(["image_path"]);
        if let Err(e) = index_writer.write_record(header.iter()) {
            error!("Error adding index header: {}", e);
            return Ok(());
        }
        if let Some(pb) = &pb {
            pb.inc(1);
        }
    }

    // Set up the communication
    let (work_tx, work_rx) = bounded::<InputRecord>(args.worker_count);
    let (save_tx, save_rx) = bounded::<OutputRecord>(args.worker_count);

    // Launch the producer
    launch_producer(source_reader, work_tx, &pb);

    // Launch the saver
    launch_saver(index_writer, missing_writer, save_rx);

    // Launch the workers
    launch_workers(
        args.url_field,
        &args.output_root,
        &args.extension,
        &work_rx,
        &save_tx,
    );

    Ok(())
}
