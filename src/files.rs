use std::fs::{File, create_dir};
use std::io::{Error, Read, Write};
use std::path::Path;

pub struct FileManager;

impl FileManager {
    pub fn create(path: &Path, file_name: &str, body: &[u8]) -> Result<(), Error> {
        if !path.exists() {
            create_dir(path)?;
        }

        let mut f = File::create(path.join(file_name))?;
        f.write_all(&body)?;
        Ok(())
    }

    pub fn read(path: &Path, file_name: &str) -> Result<Vec<u8>, Error> {
        let mut file = File::open(path.join(file_name))?;
        let mut contents: Vec<u8> = Vec::new();
        let _ = file.read_to_end(&mut contents)?;
        Ok(contents)
    }
}
