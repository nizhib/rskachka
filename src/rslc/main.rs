use std::{env::args, fs::File};

use memmap2::Mmap;
use rskachka::rslc;

pub fn main() -> std::io::Result<()> {
    let mut total = 0;
    for path in args().skip(1) {
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error opening {}: {}", path, e);
                continue;
            }
        };
        let mmap = match unsafe { Mmap::map(&file) } {
            Ok(mmap) => mmap,
            Err(e) => {
                eprintln!("Error mmaping {}: {}", path, e);
                continue;
            }
        };
        let count = rslc::count_lines(&mmap);
        total += count;
        println!("{} {}", count, path);
    }
    if (args().len() - 1) > 1 {
        println!("{} total", total);
    }
    Ok(())
}
