use std::fs::{self, File};
use std::io::{Error, ErrorKind, Read, Write};
use std::path::{Component, Path, PathBuf};

pub type FileResult<T> = Result<T, FileError>;

#[allow(dead_code)]
pub enum FileError {
    InvalidPath,
    NotFound,
    PermissionDenied,
    Io(Error),
}

fn map_io_error(err: Error) -> FileError {
    match err.kind() {
        ErrorKind::NotFound => FileError::NotFound,
        ErrorKind::PermissionDenied => FileError::PermissionDenied,
        _ => FileError::Io(err),
    }
}

fn validate_file_name(file_name: &str) -> FileResult<()> {
    let path = Path::new(file_name);
    if path
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err(FileError::InvalidPath);
    }

    Ok(())
}

fn resolve_path(path: &Path, file_name: &str) -> FileResult<PathBuf> {
    validate_file_name(file_name)?;
    let canonical_path = fs::canonicalize(path).map_err(map_io_error)?;
    Ok(canonical_path.join(file_name))
}

pub struct FileManager;

impl FileManager {
    pub fn create(path: &Path, file_name: &str, body: &[u8]) -> FileResult<()> {
        let full_path = resolve_path(path, file_name)?;

        let mut f = File::create(&full_path).map_err(map_io_error)?;
        f.write_all(body).map_err(map_io_error)?;
        Ok(())
    }

    pub fn read(path: &Path, file_name: &str) -> FileResult<Vec<u8>> {
        let full_path = resolve_path(path, file_name)?;

        let mut file = File::open(full_path).map_err(map_io_error)?;
        let mut contents: Vec<u8> = Vec::new();
        file.read_to_end(&mut contents).map_err(map_io_error)?;
        Ok(contents)
    }
}
