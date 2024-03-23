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
        #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
        {
            count += if is_x86_feature_detected!("avx2") {
                unsafe { count_lines_avx2(&buf[..size]) }
            } else if is_x86_feature_detected!("sse4.2") {
                unsafe { count_lines_sse42(&buf[..size]) }
            } else {
                count_lines_scalar(&buf[..size])
            };
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
        {
            count += count_lines_scalar(&buf[..size]);
        }
    }
    Ok(count)
}

#[inline(always)]
fn count_lines_scalar(buf: &[u8]) -> usize {
    let mut count: usize = 0;
    for c in buf.iter() {
        if *c == b'\n' {
            count += 1;
        }
    }
    count
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "sse4.2")]
unsafe fn count_lines_sse42(buf: &[u8]) -> usize {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut count: usize = 0;
    let newlines = _mm_set1_epi8(b'\n' as i8);
    let mut i = 0;
    while i + 16 <= buf.len() {
        let chunk = _mm_loadu_si128(buf[i..].as_ptr() as *const __m128i);
        let eq = _mm_cmpeq_epi8(chunk, newlines);
        let mask = _mm_movemask_epi8(eq) as u32;
        count += mask.count_ones() as usize;
        i += 16;
    }
    count + count_lines_scalar(&buf[i..])
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn count_lines_avx2(buf: &[u8]) -> usize {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let mut count: usize = 0;
    let newlines = _mm256_set1_epi8(b'\n' as i8);
    let mut i = 0;
    while i + 32 <= buf.len() {
        let chunk = _mm256_loadu_si256(buf[i..].as_ptr() as *const __m256i);
        let eq = _mm256_cmpeq_epi8(chunk, newlines);
        let mask = _mm256_movemask_epi8(eq) as u32;
        count += mask.count_ones() as usize;
        i += 32;
    }
    count + count_lines_scalar(&buf[i..])
}
