use std::{
    cmp::min,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
};

pub fn count_lines(buffer: &[u8]) -> usize {
    let num_threads = num_cpus::get();
    let buffer_size = (buffer.len() + num_threads - 1) / num_threads;

    let count = Arc::new(AtomicUsize::new(0));

    thread::scope(|s| {
        for i in 0..num_threads {
            let c_count = Arc::clone(&count);
            s.spawn(move || {
                let start = i * buffer_size;
                let end = min((i + 1) * buffer_size, buffer.len());
                let t_count = count_lines_in_buffer(&buffer[start..end]);
                c_count.fetch_add(t_count, Ordering::Relaxed);
            });
        }
    });

    count.load(Ordering::Relaxed)
}

fn count_lines_in_buffer(buf: &[u8]) -> usize {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        if is_x86_feature_detected!("avx2") {
            unsafe { count_lines_in_buffer_avx2(buf) }
        } else if is_x86_feature_detected!("sse4.2") {
            unsafe { count_lines_in_buffer_sse42(buf) }
        } else {
            count_lines_in_buffer_scalar(buf)
        }
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        count_lines_in_buffer_scalar(buf)
    }
}

#[inline(always)]
fn count_lines_in_buffer_scalar(buf: &[u8]) -> usize {
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
unsafe fn count_lines_in_buffer_sse42(buf: &[u8]) -> usize {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let newlines = _mm_set1_epi8(b'\n' as i8);
    let mut count: usize = 0;
    let mut pos = 0;
    while pos + 16 <= buf.len() {
        let chunk = _mm_loadu_si128(buf[pos..].as_ptr() as *const __m128i);
        let eq = _mm_cmpeq_epi8(chunk, newlines);
        let mask = _mm_movemask_epi8(eq) as u32;
        count += mask.count_ones() as usize;
        pos += 16;
    }
    count + count_lines_in_buffer_scalar(&buf[pos..])
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
#[target_feature(enable = "avx2")]
unsafe fn count_lines_in_buffer_avx2(buf: &[u8]) -> usize {
    #[cfg(target_arch = "x86")]
    use std::arch::x86::*;
    #[cfg(target_arch = "x86_64")]
    use std::arch::x86_64::*;

    let newlines = _mm256_set1_epi8(b'\n' as i8);
    let mut count: usize = 0;
    let mut pos = 0;
    while pos + 32 <= buf.len() {
        let chunk = _mm256_loadu_si256(buf[pos..].as_ptr() as *const __m256i);
        let eq = _mm256_cmpeq_epi8(chunk, newlines);
        let mask = _mm256_movemask_epi8(eq) as u32;
        count += mask.count_ones() as usize;
        pos += 32;
    }
    count + count_lines_in_buffer_scalar(&buf[pos..])
}
