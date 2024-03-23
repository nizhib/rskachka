pub mod item;

use std::{fmt::Write, io::BufRead, io::Result};

use indicatif::{ProgressBar, ProgressFinish, ProgressState, ProgressStyle};

const BAR_TEMPLATE: &str =
    "{percent:>3}% |{wide_bar}| {human_pos}/{human_len} [{elapsed}<{eta}, {my_per_sec}]";

pub fn maybe_create_progressbar(progress: bool, len: u64) -> Option<ProgressBar> {
    if progress {
        Some(
            ProgressBar::new(len)
                .with_style(
                    ProgressStyle::with_template(BAR_TEMPLATE)
                        .unwrap()
                        .progress_chars("##-")
                        .with_key("my_per_sec", |s: &ProgressState, w: &mut dyn Write| {
                            write!(w, "{:.02}it/s", s.per_sec()).unwrap()
                        }),
                )
                .with_finish(ProgressFinish::Abandon),
        )
    } else {
        None
    }
}

const BUFFER_SIZE: usize = 16 * 1024;

pub fn count_lines<R: BufRead>(mut reader: R) -> Result<usize> {
    let mut buf = [0u8; BUFFER_SIZE];
    let mut count: usize = 0;
    loop {
        let size = reader.read(&mut buf)?;
        if size == 0 {
            break;
        };
        for c in buf.iter() {
            if *c == b'\n' {
                count += 1;
            }
        }
    }
    Ok(count)
}
