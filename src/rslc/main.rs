use std::{env::args, fs::File, io::BufReader};

use rskachka::count_lines;

pub fn main() -> std::io::Result<()> {
    for path in args().skip(1) {
        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("Error opening {}: {}", path, e);
                continue;
            }
        };
        let count = match count_lines(BufReader::new(file)) {
            Ok(lines) => lines,
            Err(e) => {
                eprintln!("Error reading {}: {}", path, e);
                continue;
            }
        };
        println!("{:?} {}", count, path);
    }
    Ok(())
}
