use std::io::{self, Write};
use std::path::Path;
use std::fs::{File, OpenOptions};

use writer::Writer;

pub struct FileWriter {
    file: File
}

impl FileWriter {
    pub fn new(path: &Path) -> Result<FileWriter, io::Error> {
        let file = match OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            Err(e) => return Err(e),
            Ok(file) => file
        };

        Ok(FileWriter {file})
    }
}

impl Writer for FileWriter {
    fn append(&mut self, string: &str) {
        self.file.write(string.as_bytes()).unwrap();
    }
}
