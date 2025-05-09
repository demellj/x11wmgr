use crate::windowmanager::{Window, ZIndexType};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WinResize {
    pub id: Window,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WinMove {
    pub id: Window,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WinVisbilty {
    pub id: Window,
    pub visible: bool,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WinZIndex {
    pub id: Window,
    pub zindex: ZIndexType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Request {
    ChangeVisibility(Vec<WinVisbilty>),
    ChangeZIndex(Vec<WinZIndex>),
    ListNewWindows,

    ResizeWindows(Vec<WinResize>),
    MoveWindows(Vec<WinMove>),
    ListVisibleWindows,
    ListHiddenWindows,
    FocusWindow(Window),
    Commit,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct WindowInfo {
    pub id: Window,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum Response {
    VisibiltyChanged(Vec<Window>),
    ZIndexChanged(Vec<Window>),
    NewWindows(Vec<WindowInfo>),
    VisibleWindows(Vec<WindowInfo>),
    HiddenWindows(Vec<WindowInfo>),
    CommitComplete,
    MoveComplete,
    ResizeComplete,
    WindowFocused(bool),
}
