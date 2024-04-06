mod args;

use std::fs::File;

use clap::Parser;
use log::warn;
use memmap2::Mmap;
use rskachka::{item::Item, maybe_create_progressbar, rslc};

use crate::args::Args;

pub fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let source_file = match File::open(&args.source_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening source file: {}", e);
            return Ok(());
        }
    };
    let source_mmap = match unsafe { Mmap::map(&source_file) } {
        Ok(mmap) => mmap,
        Err(e) => {
            eprintln!("Error mmaping source file: {}", e);
            return Ok(());
        }
    };
    let source_lines = rslc::count_lines(&source_mmap);

    let mut source_reader = csv::ReaderBuilder::new()
        .has_headers(!args.no_header)
        .from_path(args.source_path)?;

    let header = if args.no_header {
        None
    } else {
        Some(source_reader.headers()?)
    };

    let index_file = match File::create(&args.index_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error creating index file: {}", e);
            return Ok(());
        }
    };
    let mut index_writer = csv::Writer::from_writer(index_file);

    let mut missing_writer = if args.missing_path.is_empty() {
        None
    } else {
        let missing_file = match File::create(&args.missing_path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error creating missing file: {}", e);
                return Ok(());
            }
        };
        Some(csv::Writer::from_writer(missing_file))
    };

    if let Some(header) = header {
        if missing_writer.is_some() {
            if let Err(e) = missing_writer.as_mut().unwrap().write_record(header.iter()) {
                eprintln!("Error adding missing header: {}", e);
                return Ok(());
            }
        }
        let mut header = header.to_owned();
        header.extend(["path"]);
        if let Err(e) = index_writer.write_record(header.iter()) {
            eprintln!("Error adding index header: {}", e);
            return Ok(());
        }
    }

    let pb = maybe_create_progressbar(args.progress, source_lines as u64);

    for record in source_reader.records() {
        match record {
            Ok(mut record) => {
                match Item::from_record(&record, &[0], args.url_field, &args.output_root) {
                    Ok(item) => {
                        if !item.path.exists() {
                            if let Some(missing_writer) = &mut missing_writer {
                                if let Err(e) = missing_writer.write_record(record.iter()) {
                                    warn!("Error adding missing record: {}", e);
                                };
                            }
                        } else {
                            record.extend([item.path.to_str().unwrap()]);
                            if let Err(e) = index_writer.write_record(record.iter()) {
                                warn!("Error adding index record: {}", e);
                            };
                        }
                    }
                    Err(e) => {
                        warn!("Error parsing record: {}", e);
                    }
                }
            }
            Err(e) => {
                warn!("Error reading record: {}", e);
            }
        }
        if let Some(pb) = &pb {
            pb.inc(1);
        }
    }

    Ok(())
}
