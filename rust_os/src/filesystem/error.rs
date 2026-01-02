use core::fmt;
use no_std_io::io;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Error {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    Interrupted,
    Other,
    UnexpectedEof,
    Uncategorized,
    MountFailed,
    UnexpectedFileType,
}

impl Error {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Error::NotFound => "entity not found",
            Error::PermissionDenied => "permission denied",
            Error::ConnectionRefused => "connection refused",
            Error::ConnectionReset => "connection reset",
            Error::ConnectionAborted => "connection aborted",
            Error::NotConnected => "not connected",
            Error::AddrInUse => "address in use",
            Error::AddrNotAvailable => "address not available",
            Error::BrokenPipe => "broken pipe",
            Error::AlreadyExists => "entity already exists",
            Error::WouldBlock => "operation would block",
            Error::InvalidInput => "invalid input parameter",
            Error::InvalidData => "invalid data",
            Error::TimedOut => "timed out",
            Error::WriteZero => "write zero",
            Error::Interrupted => "operation interrupted",
            Error::Other => "other os error",
            Error::UnexpectedEof => "unexpected end of file",
            Error::Uncategorized => "uncategorized",
            Error::MountFailed => "filesystem mount failed",
            Error::UnexpectedFileType => "unexpected filesystem entry type",
        }
    }
}

impl From<io::ErrorKind> for Error {
    fn from(kind: io::ErrorKind) -> Self {
        match kind {
            io::ErrorKind::NotFound => Error::NotFound,
            io::ErrorKind::PermissionDenied => Error::PermissionDenied,
            io::ErrorKind::ConnectionRefused => Error::ConnectionRefused,
            io::ErrorKind::ConnectionReset => Error::ConnectionReset,
            io::ErrorKind::ConnectionAborted => Error::ConnectionAborted,
            io::ErrorKind::NotConnected => Error::NotConnected,
            io::ErrorKind::AddrInUse => Error::AddrInUse,
            io::ErrorKind::AddrNotAvailable => Error::AddrNotAvailable,
            io::ErrorKind::BrokenPipe => Error::BrokenPipe,
            io::ErrorKind::AlreadyExists => Error::AlreadyExists,
            io::ErrorKind::WouldBlock => Error::WouldBlock,
            io::ErrorKind::InvalidInput => Error::InvalidInput,
            io::ErrorKind::InvalidData => Error::InvalidData,
            io::ErrorKind::TimedOut => Error::TimedOut,
            io::ErrorKind::WriteZero => Error::WriteZero,
            io::ErrorKind::Interrupted => Error::Interrupted,
            io::ErrorKind::Other => Error::Other,
            io::ErrorKind::UnexpectedEof => Error::UnexpectedEof,
            io::ErrorKind::Uncategorized => Error::Uncategorized,
            _ => Error::Other,
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::from(err.kind())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
