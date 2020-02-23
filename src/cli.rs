use std::io;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::{de, ser};

use crate::error::*;
use crate::windowmanager::{Waker, WINDOW};

#[derive(Serialize, Deserialize, Debug)]
pub struct WinVisbilty {
    pub id: WINDOW,
    pub visible: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct WinZIndex {
    pub id: WINDOW,
    pub zindex: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    ChangeVisiblity(Vec<WinVisbilty>),
    ChangeZIndex(Vec<WinZIndex>),
    ListNewWindows,
    ListVisibleWindows,
    ListHiddenWindows,
    RestackWindows,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    VisibiltyChanged(Vec<WINDOW>),
    ZIndexChanged(Vec<WINDOW>),
    NewWindows(Vec<WINDOW>),
    VisibleWindows(Vec<WinZIndex>),
    HiddenWindows(Vec<WinZIndex>),
    RestackComplete,
}

#[derive(Serialize, Deserialize, Debug)]
enum ErrorType {
    InvalidInput(String),
}

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    Error(ErrorType),
    Response(Response),
}

pub fn create_cli(waker: Waker, tx_req: Sender<Request>) -> Result<Sender<Response>, Error> {
    let (tx_resp, rx_resp) = channel::<Response>();

    thread::spawn(move || {
        let mut line = String::new();

        while io::stdin().read_line(&mut line).is_ok() {
            if let Ok(req) = de::from_str::<Request>(&line) {
                if tx_req.send(req).is_ok() {
                    // wake up wm thread, notifying it of pending input
                    let res = waker.wake();
                    if res.is_ok() {
                        if let Ok(resp) = rx_resp.recv() {
                            if let Ok(resp) = ser::to_string(&Message::Response(resp)) {
                                println!("{}", resp);
                            }
                        }
                    } else {
                        eprintln!("{}", res.err().unwrap());
                    }
                }
            } else {
                let msg = line.trim().to_owned();
                let resp = Message::Error(ErrorType::InvalidInput(msg));
                println!("{}", ser::to_string(&resp).unwrap());
            }
            line.clear();
        }
    });

    Ok(tx_resp)
}
