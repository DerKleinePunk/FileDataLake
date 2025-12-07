use std::collections::HashMap;
use uuid::{Uuid};

#[derive(Debug)]
pub struct FileEntry {
    pub id: Uuid,
    pub name: String,
    pub size: u64,
    pub hash: String,
    pub attributes: HashMap<String,String>
}

impl FileEntry {
    pub fn new() -> FileEntry {
        let id = Uuid::now_v7();
        FileEntry { id: id, name : String::new(), size: 0, hash: String::new(), attributes: HashMap::new()}
    }
}
