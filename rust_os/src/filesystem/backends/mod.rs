mod tar;

use alloc::{borrow::Cow, vec::Vec};

use crate::filesystem::{FileMetadata, Result, backends::tar::TarBackend, path::CanonPathString};

pub enum FsBackendImpl {
    Tar(tar::TarBackend)
}

impl FsBackendImpl {
    pub fn from_tar(buffer: Cow<'static, [u8]>) -> Result<Self> {
        let tar_backend = TarBackend::new(buffer)?;
        Ok(FsBackendImpl::Tar(tar_backend))
    }

    pub fn read_into(&mut self, path: &CanonPathString, position:usize, buffer: &mut[u8]) -> Result<usize> {
        match self {
           FsBackendImpl::Tar(b) => b.read_into(path, position, buffer) 
        }
    }

    pub fn file_metadata(&self, path: &CanonPathString) -> Result<FileMetadata> {
        match self {
           FsBackendImpl::Tar(b) => b.file_metadata(path) 
        }
    }

    pub fn read_dir(&self, path: &CanonPathString) -> Result<Vec<FileMetadata>> {
        match self {
            FsBackendImpl::Tar(b) => b.read_dir(path)
        }
    }
}

