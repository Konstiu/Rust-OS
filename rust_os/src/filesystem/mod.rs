use alloc::{borrow::Cow, string::String, vec::Vec};
use tarfs::{Entity, TarFS, Type};

mod error;
mod tar;

pub use error::{Error, Result};

pub struct FileSystem {
    inner: TarFS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    File,
    HardLink,
    SymLink,
    Dir,
}

impl TryFrom<Type> for FileType {
    type Error = Error;

    fn try_from(value: Type) -> Result<Self> {
        match value {
           Type::File => Ok(FileType::File),
           Type::HardLink => Ok(FileType::HardLink),
           Type:: SymbLink => Ok(FileType::SymLink),
           Type::Dir => Ok(FileType::Dir),
           _ => Err(Error::UnexpectedFileType),
        }
    }
}

pub struct FileMetadata {
    pub name: String,
    pub size: usize,
    pub file_type: FileType 
}

impl TryFrom<Entity> for FileMetadata {
    type Error = Error;

    fn try_from(value: Entity) -> Result<Self> {
        let file_type = value._type.try_into()?;
        Ok(FileMetadata { 
            name: value.name,
            size: value.size,
            file_type,
        })
    }
}

impl FileSystem {

    pub fn from_tar(buffer: Cow<'static, [u8]>) -> Result<FileSystem> {
        let tar_fs = tar::create_tar_fs(buffer)?;
        let file_system = FileSystem {
            inner: tar_fs
        };
        Ok(file_system)
    } 

    pub fn read(&mut self, path: &str) -> Result<Vec<u8>> {
        let data = self.inner.read_entire_file(path)?;
        Ok(data)
    }

    pub fn read_to_string(&mut self, path: &str) -> Result<String> {
        let data = self.inner.read_to_string(path)?;
        Ok(data)
    }

    pub fn read_into(&mut self, path: &str, position: usize, buffer: &mut [u8]) -> Result<usize> {
        let bytes_read = self.inner.read_file(path, position, buffer)?;
        Ok(bytes_read)
    }

    pub fn read_dir(&mut self, path: &str) -> Result<Vec<FileMetadata>> {
        let entries = self.inner.list_by_path_shallow(path)?;
        let data = entries
            .into_iter()
            .map(FileMetadata::try_from)
            .collect::<Result<Vec<_>>>()?;
        Ok(data)
    }
    
}
