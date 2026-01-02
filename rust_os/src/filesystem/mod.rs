use alloc::borrow::Cow;
use no_std_io::io::{self, Cursor};
use tarfs::TarFS;

struct InMemoryDevice<T> {
    cursor: Cursor<T>
}

impl<T> InMemoryDevice<T> {
    fn new(inner: T) -> Self {
        InMemoryDevice { cursor: Cursor::new(inner) }
    }
}

impl<T: AsRef<[u8]>> io::Read for InMemoryDevice<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.cursor.read(buf)
    }
}

impl<T: AsRef<[u8]>> io::Seek for InMemoryDevice<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.cursor.seek(pos)
    }
}

impl<T: AsRef<[u8]>> tarfs::Device for InMemoryDevice<T> {}    


pub fn create_tarfs(buffer: impl Into<Cow<'static, [u8]>>) -> TarFS {
    TarFS::from_device(InMemoryDevice::new(buffer.into()))
        .expect("Could not create TarFS")
}


