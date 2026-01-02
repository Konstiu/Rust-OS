use alloc::borrow::Cow;
use no_std_io::io::{self, Cursor};
use tarfs::TarFS;

use super::{Error, Result};

trait BytesLike: AsRef<[u8]> {}
impl<T: AsRef<[u8]>> BytesLike for T {}

struct TarFsDevice<T: BytesLike> {
    inner: Cursor<T> 
}

impl<T: BytesLike> TarFsDevice<T> {
    fn new(buffer: T) -> Self {
        TarFsDevice{ 
            inner: Cursor::new(buffer)
        }
    }
}

impl <T: BytesLike> io::Read for TarFsDevice<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}

impl <T: BytesLike> io::Seek for TarFsDevice<T> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

impl <T: BytesLike> tarfs::Device for TarFsDevice<T> {}

pub fn create_tar_fs(buffer: impl Into<Cow<'static, [u8]>>) -> Result<TarFS> {
    TarFS::from_device(TarFsDevice::new(buffer.into())).ok_or(Error::MountFailed)
}
