use alloc::{borrow::Cow, collections::btree_map::BTreeMap, vec::Vec, string::String};
use no_std_io::io::{Cursor, Read, Seek};
use tarfs::{TarFS, Type, Entity};

use crate::filesystem::{Error, FileMetadata, FileType, Result, path::CanonPathString};

pub struct TarBackend {
    tarfs: TarFS,
    entries: BTreeMap<CanonPathString, Entity>
}


impl TarBackend {
    pub fn new(buffer: Cow<'static, [u8]>) -> Result<Self> {
        let mut tarfs = TarFS::from_device(TarFsDevice { cursor: Cursor::new(buffer)}).ok_or(Error::MountFailed)?;
        let mut entries: BTreeMap<CanonPathString, Entity> = BTreeMap::new();
        for entity in tarfs.list() {
            let entity = entity?;
            let canonicalized_name: CanonPathString = entity.name.as_str().try_into()?;
            entries.insert(canonicalized_name, entity);
        }
        Ok(TarBackend { tarfs, entries })
    }

    pub fn read_into(&mut self, path: &CanonPathString, position: usize, buffer: &mut[u8]) -> Result<usize> {
        let file_entity = self.entries.get(path).ok_or(Error::NotFound)?;
        let bytes_read = self.tarfs.read_file_by_entity(&file_entity, position, buffer)?;
        Ok(bytes_read)
    }

    pub fn file_metadata(&self, path: &CanonPathString) -> Result<FileMetadata> {
        let file_entry= self.entries.get_key_value(path).ok_or(Error::NotFound)?;
        let file_metadata = file_entry.try_into()?;
        Ok(file_metadata)
    }

    pub fn read_dir(&self, path: &CanonPathString) -> Result<Vec<FileMetadata>> {
        let entries = self.entries.iter()
            .filter(|e| is_immediate_child(path.as_str(), e.0.as_str()))
            .map(|e| FileMetadata::try_from(e))
            .collect::<Result<Vec<_>>>()?;

        Ok(entries)
    }
}

fn is_immediate_child(dir: &str, path: &str) -> bool {
    let path = path.trim_end_matches('/');
    if dir.is_empty() {
        return ! path.is_empty() && ! path.contains('/');
    }

    if let Some(rest) = path.strip_prefix(dir).and_then(|r| r.strip_prefix('/')) {
        let rest = rest.trim_end_matches('/');
        ! rest.is_empty() && !rest.contains('/')
    } else {
        false
    }
}

struct TarFsDevice<T: AsRef<[u8]>> {
    cursor: Cursor<T>
}

impl <T: AsRef<[u8]>> Read for TarFsDevice<T> {
    fn read(&mut self, buf: &mut [u8]) -> no_std_io::io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl <T: AsRef<[u8]>> Seek for TarFsDevice<T> {
    fn seek(&mut self, pos: no_std_io::io::SeekFrom) -> no_std_io::io::Result<u64> {
        self.cursor.seek(pos)
    }
}

impl <T: AsRef<[u8]>> tarfs::Device for TarFsDevice<T> {}

impl TryFrom<&Type> for FileType {
    type Error = Error;

    fn try_from(value: &Type) -> Result<Self> {
        match value {
           Type::File => Ok(FileType::File),
           Type::HardLink => Ok(FileType::HardLink),
           Type:: SymbLink => Ok(FileType::SymLink),
           Type::Dir => Ok(FileType::Dir),
           _ => Err(Error::UnexpectedFileType),
        }
    }
}

impl TryFrom<(&CanonPathString, &Entity)> for FileMetadata {
    type Error = Error;
    
    fn try_from(value: (&CanonPathString, &Entity)) -> core::result::Result<Self, Self::Error> {
        let canon_path = value.0;
        let entity = value.1;

        let file_type = (&entity._type).try_into()?;
        let path: String = canon_path.as_str().into();

        Ok(FileMetadata { 
            path,
            size: entity.size,
            file_type,
        })
    }
}