use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::{de, ser};

use crate::error::*;
use crate::windowmanager::{Waker, Window, ZIndexType};

#[derive(Serialize, Deserialize, Debug)]
pub struct WinResize {
    pub id: Window,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WinMove {
    pub id: Window,
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WinMoveResize {
    pub id: Window,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WinVisbilty {
    pub id: Window,
    pub visible: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WinZIndex {
    pub id: Window,
    pub zindex: ZIndexType,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    ChangeVisibility(Vec<WinVisbilty>),
    ChangeZIndex(Vec<WinZIndex>),
    ListNewWindows,
    MoveResizeWindows(Vec<WinMoveResize>),
    ResizeWindows(Vec<WinResize>),
    MoveWindows(Vec<WinMove>),
    ListVisibleWindows,
    ListHiddenWindows,
    RestackWindows,
    FocusWindow(Window),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    VisibiltyChanged(Vec<Window>),
    ZIndexChanged(Vec<Window>),
    NewWindows(Vec<Window>),
    VisibleWindows(Vec<WinZIndex>),
    HiddenWindows(Vec<WinZIndex>),
    RestackComplete,
    MoveComplete,
    ResizeComplete,
    MoveResizeComplete,
    WindowFocused(bool),
}

#[derive(Serialize, Deserialize, Debug)]
enum ErrorType {
    InvalidInput(String),
    InternalError(String),
}

#[derive(Serialize, Deserialize, Debug)]
enum ResponseEnvelope {
    Error(ErrorType),
    Result(Response),
}

pub fn create_cli(waker: Waker, tx_req: Sender<Request>) -> Sender<Response> {
    let (tx_resp, rx_resp) = channel::<Response>();

    thread::spawn(move || {
        let mut line = String::new();

        while io::stdin().read_line(&mut line).is_ok() {
            if let Ok(req) = de::from_str::<Request>(&line) {
                match handle_input(req, &tx_req, &waker, &rx_resp) {
                    Ok(resp) => {
                        let resp = ser::to_string(&ResponseEnvelope::Result(resp)).unwrap();
                        println!("{}", resp);
                    }
                    Err(err) => {
                        let msg = err.to_string();
                        let resp = ResponseEnvelope::Error(ErrorType::InternalError(msg));
                        eprintln!("{}", ser::to_string(&resp).unwrap());
                    }
                }
            } else {
                let msg = line.trim().to_owned();
                let resp = ResponseEnvelope::Error(ErrorType::InvalidInput(msg));
                eprintln!("{}", ser::to_string(&resp).unwrap());
            }
            line.clear();
        }
    });

    tx_resp
}

fn handle_input(
    req: Request,
    tx_req: &Sender<Request>,
    waker: &Waker,
    rx_resp: &Receiver<Response>,
) -> Result<Response, Error> {
    tx_req.send(req)?;
    waker.wake()?; // wake up wm thread, notifying it of pending input
    let resp = rx_resp.recv()?;
    Ok(resp)
}
