mod cli;
mod windowmanager;

pub use cli::*;
pub use windowmanager::{ConnectionErrorOrX11Error, WinInfo, WindowManager};
