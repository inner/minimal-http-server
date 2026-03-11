use std::fs::{self, File, create_dir};
use std::io::{Error, ErrorKind, Read, Write};
use std::path::Path;

pub struct FileManager;

impl FileManager {
    pub fn create(path: &Path, file_name: &str, body: &[u8]) -> Result<(), Error> {
        if file_name.split('/').any(|s| s == "..") {
            return Err(Error::new(ErrorKind::PermissionDenied, "invalid path"));
        }

        if !path.exists() {
            create_dir(path)?;
        }

        let mut f = File::create(path.join(file_name))?;
        f.write_all(&body)?;
        Ok(())
    }

    pub fn read(path: &Path, file_name: &str) -> Result<Vec<u8>, Error> {
        let full_path = path.join(file_name);
        let canonical = fs::canonicalize(&full_path)?;

        if !canonical.starts_with(path) {
            return Err(Error::new(ErrorKind::PermissionDenied, "invalid path"));
        }

        let mut file = File::open(canonical)?;
        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }
}
