use std::sync::Arc;

use crate::{dtype::UnknownDType, endian::InvalidSignature, version::InvalidVersion};

/// An error that can occur in this crate.
#[derive(Clone)]
pub struct Error {
    inner: Arc<ErrorInner>,
}

struct ErrorInner {
    kind: ErrorKind,
    cause: Option<Error>,
}

/// The underlying kind of [`Error`].
#[derive(Debug)]
enum ErrorKind {
    /// An error that is constructed from anything that implements [`std::fmt::Display`].
    AdHoc(Box<str>),
    /// An error that occurred while reading or writing.
    Io(std::io::Error),
    /// The image signature is invalid.
    InvalidSignature(InvalidSignature),
    /// The image version is not valid.
    InvalidVersion(InvalidVersion),
    /// An unknown datatype was encountered.
    UnknownDType(UnknownDType),
}

impl Error {
    /// Constructs a new [`Error`] value from [`std::fmt::Arguments`].
    #[inline(always)]
    pub(crate) fn from_args(args: std::fmt::Arguments<'_>) -> Self {
        Error::from(ErrorKind::AdHoc(args.to_string().into_boxed_str()))
    }

    /// Constructs a new [`Error`] value from a `&'static str`.
    #[inline(always)]
    pub(crate) fn from_static_str(msg: &'static str) -> Self {
        Error::from(ErrorKind::AdHoc(msg.into()))
    }
}

impl std::error::Error for Error {}

impl std::fmt::Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !f.alternate() {
            return std::fmt::Display::fmt(self, f);
        }

        f.debug_struct("Error")
            .field("kind", &self.inner.kind)
            .field("cause", &self.inner.cause)
            .finish()
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut err = self;
        loop {
            write!(f, "{}", err.inner.kind)?;
            err = match &err.inner.cause {
                Some(err) => err,
                None => break,
            };
            write!(f, ": ")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::AdHoc(msg) => write!(f, "{msg}"),
            ErrorKind::Io(err) => err.fmt(f),
            ErrorKind::InvalidSignature(err) => err.fmt(f),
            ErrorKind::InvalidVersion(err) => err.fmt(f),
            ErrorKind::UnknownDType(err) => err.fmt(f),
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            inner: Arc::new(ErrorInner { kind, cause: None }),
        }
    }
}

impl From<std::io::Error> for Error {
    #[inline(always)]
    fn from(err: std::io::Error) -> Self {
        Error::from(ErrorKind::Io(err))
    }
}

impl From<InvalidSignature> for Error {
    #[inline(always)]
    fn from(err: InvalidSignature) -> Self {
        Error::from(ErrorKind::InvalidSignature(err))
    }
}

impl From<InvalidVersion> for Error {
    #[inline(always)]
    fn from(err: InvalidVersion) -> Self {
        Error::from(ErrorKind::InvalidVersion(err))
    }
}

impl From<UnknownDType> for Error {
    #[inline(always)]
    fn from(err: UnknownDType) -> Self {
        Error::from(ErrorKind::UnknownDType(err))
    }
}
