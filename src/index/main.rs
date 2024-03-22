mod args;

use clap::Parser;
use log::warn;
use rskachka::{item::Item, maybe_create_progressbar};

use crate::args::Args;

pub fn main() -> std::io::Result<()> {
    let args = Args::parse();

    let mut csv_reader = csv::ReaderBuilder::new()
        .has_headers(!args.no_header)
        .from_path(args.index_path)?;

    let mut csv_writer = csv::Writer::from_path(args.output_path)?;

    let pb = maybe_create_progressbar(args.progress, 0);

    for record in csv_reader.records() {
        match record {
            Ok(mut record) => match Item::from_record(&record, &[0], args.url_field, &args.root) {
                Ok(item) => {
                    record.extend([item.path.to_str().unwrap()]);
                    match csv_writer.write_record(record.iter()) {
                        Ok(_) => {}
                        Err(e) => {
                            warn!("Error writing record: {}", e);
                        }
                    };
                }
                Err(e) => {
                    warn!("Error parsing record: {}", e);
                }
            },
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