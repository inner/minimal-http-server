use std::fs::{self, File};
use std::io::{Error, ErrorKind, Read, Write};
use std::path::Path;

pub struct FileManager;

impl FileManager {
    pub fn create(path: &Path, file_name: &str, body: &[u8]) -> Result<(), Error> {
        let canonical_dir = fs::canonicalize(path)?;

        let p = Path::new(file_name);
        if p.components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            return Err(Error::new(ErrorKind::PermissionDenied, "invalid path"));
        }

        let full_path = canonical_dir.join(file_name);

        let mut f = File::create(&full_path)?;
        f.write_all(body)?;
        Ok(())
    }

    pub fn read(path: &Path, file_name: &str) -> Result<Vec<u8>, Error> {
        let canonical_dir = fs::canonicalize(path)?;
        let full_path = fs::canonicalize(path.join(file_name))?;

        if !full_path.starts_with(&canonical_dir) {
            return Err(Error::new(ErrorKind::PermissionDenied, "invalid path"));
        }

        let mut file = File::open(full_path)?;
        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut contents)?;
        Ok(contents)
    }
}
