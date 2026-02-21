use std::fs::{File, create_dir};
use std::io::{Error, Write};
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
}
