pub mod item;
pub mod rslc;

use std::fmt::Write;

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
