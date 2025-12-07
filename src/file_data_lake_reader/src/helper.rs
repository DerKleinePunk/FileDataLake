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
