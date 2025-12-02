use std::{path::Path};
use std::fs;
use std::io;

pub fn print_file_size<P: AsRef<Path>>(path : P) -> io::Result<u64> {
    log::debug!("print_file_size");

    let metadata = fs::metadata(path)?;
    //let attributes = metadata.file_attributes();
    let size = metadata.len();

    log::debug!("print_file_size {size:?}");

    Ok(size)
}
