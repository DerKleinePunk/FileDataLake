use std::path::PathBuf;
use std::fs;
use std::io;

pub fn print_file_size(path : &PathBuf) -> io::Result<u64> {
    log::debug!("print_file_size {path:?}");

    let metadata = fs::metadata(path)?;
    //let attributes = metadata.file_attributes();
    let size = metadata.len();

    log::debug!("print_file_size {size:?}");

    Ok(size)
}

pub fn new_file_flow(path : &PathBuf) -> Result<(), String> {
    log::debug!("print_file_size {path:?}");

    Ok(())
}
