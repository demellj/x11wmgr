use failure::{Backtrace, Context, Fail};
use std::fmt::{Display, Formatter, Result as DisplayResult};
use std::sync::Arc;

use std::io::Error as IOError;
use std::sync::mpsc::SendError;
use x11rb::errors::ConnectionError;
use x11rb::errors::ConnectionErrorOrX11Error;
use x11rb::errors::ParseError;
use x11rb::generated::xproto::ACCESS_ERROR;
use x11rb::x11_utils::GenericError;

use crate::cli::Response;

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "An X11 error occurred")]
    X11Error(#[cause] ConnectionErrorOrX11Error),

    #[fail(display = "XCB connection failed")]
    XCBError(#[cause] ConnectionError),

    #[fail(display = "An X11 error occurred")]
    GenericX11Error(GenericError),

    #[fail(display = "An IO error occurred")]
    IOError(#[cause] IOError),

    #[fail(display = "An internal error occurred")]
    ParseError(#[cause] ParseError),

    #[fail(display = "An internal error occurred")]
    SendError(#[cause] SendError<Response>),
}

#[derive(Debug, Clone)]
pub struct Error(Arc<Context<ErrorKind>>);

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> DisplayResult {
        let ctx = self.0.get_context();
        match ctx {
            ErrorKind::GenericX11Error(err) => {
                if err.error_code() == ACCESS_ERROR {
                    write!(f, "Another window manager is active")
                } else {
                    write!(f, "{}", ctx)
                }
            }
            _ => write!(f, "{}", ctx),
        }
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.0.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.0.backtrace()
    }
}

impl From<IOError> for Error {
    fn from(error: IOError) -> Self {
        Error(Arc::new(Context::new(ErrorKind::IOError(error))))
    }
}

impl From<ConnectionErrorOrX11Error> for Error {
    fn from(error: ConnectionErrorOrX11Error) -> Self {
        Error(Arc::new(Context::new(ErrorKind::X11Error(error))))
    }
}

impl From<ConnectionError> for Error {
    fn from(error: ConnectionError) -> Self {
        Error(Arc::new(Context::new(ErrorKind::XCBError(error))))
    }
}

impl From<ParseError> for Error {
    fn from(error: ParseError) -> Self {
        Error(Arc::new(Context::new(ErrorKind::ParseError(error))))
    }
}

impl From<GenericError> for Error {
    fn from(error: GenericError) -> Self {
        Error(Arc::new(Context::new(ErrorKind::GenericX11Error(error))))
    }
}

impl From<SendError<Response>> for Error {
    fn from(error: SendError<Response>) -> Self {
        Error(Arc::new(Context::new(ErrorKind::SendError(error))))
    }
}
