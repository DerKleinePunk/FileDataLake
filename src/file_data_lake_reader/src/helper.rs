use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{PathBuf};

// calculates sha256 digest as uppercase hex string
pub fn sha256_digest(path: &PathBuf) -> Result<String, std::io::Error> {
    let input = File::open(path)?;
    let mut reader = BufReader::new(input);

    let digest = {
        let mut hasher = Sha256::new();
        let mut buffer = [0; 1024];
        loop {
            let count = reader.read(&mut buffer)?;
            if count == 0 { break }
            hasher.update(&buffer[..count]);
        }
        hasher.finalize()
    };

    Ok(format!("{:X}", digest))
}

pub fn is_file_type(path: &PathBuf, ext: &str) -> bool{
    path.is_file() && path.extension().map(|s| s == ext).unwrap_or(false)
}

pub fn is_file_image(path: &PathBuf) -> bool {
    //Todo all Supported Formats im Image ...
    let mut result = is_file_type(path,"png");

    if !result {
         result = is_file_type(path,"jpg");
    }

    result
}
