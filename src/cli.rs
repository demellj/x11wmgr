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
    ListNewWindow,
    RestackWindows,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Response {
    VisibiltyChanged(Vec<WINDOW>),
    ZIndexChanged(Vec<WINDOW>),
    NewWindows(Vec<WINDOW>),
    RestackComplete
}

pub fn create_cli(tx_req: Sender<Request>) -> Sender<Response> {
    let (tx_resp, rx_resp) = channel::<Response>();

    thread::spawn(move || {
        let mut line = String::new();

        while let Ok(_) = io::stdin().read_line(&mut line) {
            if let Ok(req) = de::from_str::<Request>(line.as_ref()) {
                if tx_req.send(req).is_ok() {
                    if let Ok(resp) = rx_resp.recv() {
                        if let Ok(resp) = ser::to_string(&resp) {
                            println!("{}", resp);
                        }
                    }
                }
            }
        }
    });

    tx_resp
}
