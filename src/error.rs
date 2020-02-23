use failure::{Backtrace, Context, Fail};
use std::fmt::{Display, Formatter, Result as DisplayResult};
use std::sync::Arc;

use std::io::Error as IOError;
use x11rb::errors::ConnectionErrorOrX11Error;

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "An X11 error occurred")]
    X11Error(#[cause] ConnectionErrorOrX11Error),

    #[fail(display = "An IO error occurred")]
    IOError(#[cause] IOError),
}

#[derive(Debug, Clone)]
pub struct Error {
    ctx: Arc<Context<ErrorKind>>,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> DisplayResult {
        write!(f, "{}", self.ctx.get_context())
    }
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.ctx.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.ctx.backtrace()
    }
}

impl From<IOError> for Error {
    fn from(error: IOError) -> Self {
        Error {
            ctx: Arc::new(Context::new(ErrorKind::IOError(error))),
        }
    }
}

impl From<ConnectionErrorOrX11Error> for Error {
    fn from(error: ConnectionErrorOrX11Error) -> Self {
        Error {
            ctx: Arc::new(Context::new(ErrorKind::X11Error(error))),
        }
    }
}