use std::fmt::{Display, Formatter, Result as DisplayResult};
use std::sync::Arc;
use thiserror::Error;
use x11rb::rust_connection::ReplyOrIdError;

use std::io::Error as IOError;
use std::sync::mpsc::{RecvError, SendError};
use x11rb::errors::ConnectError;
use x11rb::errors::ConnectionError;
use x11rb::errors::ParseError;
use x11rb::errors::ReplyError;
use x11rb::protocol::xproto::ACCESS_ERROR;
use x11rb::x11_utils::X11Error;

use crate::messages::{Request, Response};

#[derive(Debug, Error)]
pub enum ErrorKind {
    #[error("Failed to connect to server")]
    ConnectError(#[from] ConnectError),

    #[error("Connection terminated")]
    ConnectionError(#[from] ConnectionError),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(&'static str),

    #[error("An X11 error occurred")]
    X11Error(X11Error),

    #[error("An IO error occurred")]
    IOError(#[from] IOError),

    #[error("An internal error occurred")]
    ParseError(#[from] ParseError),

    #[error("An internal error occurred")]
    SendResponseError(#[from] SendError<Response>),

    #[error("An internal error occurred")]
    SendRequestError(#[from] SendError<Request>),

    #[error("An internal error occurred")]
    RecvError(#[from] RecvError),
}

#[derive(Debug, Clone)]
pub struct Error(Arc<ErrorKind>);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> DisplayResult {
        let ctx = &self.0.as_ref();
        match ctx {
            ErrorKind::X11Error(err) => {
                if err.error_code == ACCESS_ERROR {
                    write!(f, "Another window manager is active")
                } else {
                    write!(f, "{}", ctx)
                }
            }
            _ => write!(f, "{}", ctx),
        }
    }
}

impl std::error::Error for Error {}

impl From<IOError> for Error {
    fn from(error: IOError) -> Self {
        Error(Arc::new(ErrorKind::IOError(error)))
    }
}

impl From<ReplyError> for Error {
    fn from(error: ReplyError) -> Self {
        match error {
            ReplyError::X11Error(err) => Error(Arc::new(ErrorKind::X11Error(err))),
            ReplyError::ConnectionError(err) => Error(Arc::new(ErrorKind::ConnectionError(err))),
        }
    }
}

impl From<ReplyOrIdError> for Error {
    fn from(error: ReplyOrIdError) -> Self {
        match error {
            ReplyOrIdError::IdsExhausted => Error(Arc::new(ErrorKind::ResourceExhausted("XID"))),
            ReplyOrIdError::X11Error(err) => Error(Arc::new(ErrorKind::X11Error(err))),
            ReplyOrIdError::ConnectionError(err) => {
                Error(Arc::new(ErrorKind::ConnectionError(err)))
            }
        }
    }
}

impl From<ConnectError> for Error {
    fn from(error: ConnectError) -> Self {
        Error(Arc::new(ErrorKind::ConnectError(error)))
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error(Arc::new(ErrorKind::ParseError(error)))
    }
}

impl From<X11Error> for Error {
    fn from(error: X11Error) -> Self {
        Error(Arc::new(ErrorKind::X11Error(error)))
    }
}

impl From<SendError<Response>> for Error {
    fn from(error: SendError<Response>) -> Self {
        Error(Arc::new(ErrorKind::SendResponseError(error)))
    }
}

impl From<SendError<Request>> for Error {
    fn from(error: SendError<Request>) -> Self {
        Error(Arc::new(ErrorKind::SendRequestError(error)))
    }
}

impl From<RecvError> for Error {
    fn from(error: RecvError) -> Self {
        Error(Arc::new(ErrorKind::RecvError(error)))
    }
}

impl From<ConnectionError> for Error {
    fn from(error: ConnectionError) -> Self {
        Error(Arc::new(ErrorKind::ConnectionError(error)))
    }
}
