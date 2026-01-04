use alloc::{borrow::Cow, string::String, vec::Vec};

mod backends;
mod error;
mod path;

pub use error::{Error, Result};

use crate::filesystem::{backends::FsBackendImpl, path::CanonPathString};
use alloc::vec;

pub struct FileSystem {
    backend: FsBackendImpl
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    HardLink,
    SymLink,
    Dir,
}

pub struct FileMetadata {
    pub path: String,
    pub size: usize,
    pub file_type: FileType 
}

impl FileMetadata {
    pub fn name(&self) -> &str {
        let path = self.path.trim_end_matches('/');
        path.rsplit('/').next().unwrap_or("")
    }
}

impl FileSystem {

    pub fn from_tar(buffer: Cow<'static, [u8]>) -> Result<FileSystem> {
        let backend = FsBackendImpl::from_tar(buffer)?;
        Ok(FileSystem { backend })
    } 

    pub fn read(&mut self, path: &str) -> Result<Vec<u8>> {
        let canonicalized_path: CanonPathString = path.try_into()?;
        let metadata = self.backend.file_metadata(&canonicalized_path)?;
        let mut buffer = vec![0u8; metadata.size];
        let bytes_read = self.read_into_buffer(&canonicalized_path, 0, &mut buffer)?;
        buffer.truncate(bytes_read);
        Ok(buffer)
    }

    pub fn read_to_string(&mut self, path: &str) -> Result<String> {
        let bytes = self.read(path)?;
        let string = String::from_utf8(bytes)?;
        Ok(string)
    }

    pub fn read_into(&mut self, path: &str, position: usize, buffer: &mut [u8]) -> Result<usize> {
        let canonicalized_path: CanonPathString = path.try_into()?; 
        self.read_into_buffer(&canonicalized_path, position, buffer)
    }

    pub fn read_dir(&self, path: &str) -> Result<Vec<FileMetadata>> {
        let canonicalized_path: CanonPathString = path.try_into()?;
        let entries = self.backend.read_dir(&canonicalized_path)?;
        Ok(entries)
    }

    fn read_into_buffer(&mut self, canonicalized_path: &CanonPathString, position: usize, buffer: &mut[u8]) -> Result<usize> {
        let bytes_read = self.backend.read_into(&canonicalized_path, position, buffer)?;
        Ok(bytes_read)
    }
}
