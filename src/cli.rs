use std::io;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_json::{de, ser};

use crate::windowmanager::WINDOW;

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
    RestackWindows,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    VisibiltyChanged(Vec<WINDOW>),
    ZIndexChanged(Vec<WINDOW>),
    NewWindows(Vec<WINDOW>),
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

pub fn create_cli(tx_req: Sender<Request>) -> Sender<Response> {
    let (tx_resp, rx_resp) = channel::<Response>();

    thread::spawn(move || {
        let mut line = String::new();

        while io::stdin().read_line(&mut line).is_ok() {
            if let Ok(req) = de::from_str::<Request>(&line) {
                if tx_req.send(req).is_ok() {
                    if let Ok(resp) = rx_resp.recv() {
                        if let Ok(resp) = ser::to_string(&Message::Response(resp)) {
                            println!("{}", resp);
                        }
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

    tx_resp
}
