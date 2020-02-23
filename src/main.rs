use std::sync::mpsc::channel;
use std::thread::sleep;
use std::time::{Duration, Instant};

use x11wmgr::*;

fn main() -> Result<(), Error> {
    let min_wait = Duration::from_millis(100);

    let mut wm = WindowManager::new()?;

    let (tx_req, rx_req) = channel::<Request>();
    let tx_resp = create_cli(tx_req);

    loop {
        let start = Instant::now();
        wm.process_events()?;

        if let Ok(req) = rx_req.try_recv() {
            match req {
                Request::ChangeVisiblity(win_vis) => {
                    let result =
                        wm.change_visiblity(win_vis.into_iter().map(|v| (v.id, v.visible)));
                    tx_resp.send(Response::VisibiltyChanged(result)).unwrap();
                }
                Request::ChangeZIndex(win_indices) => {
                    let result =
                        wm.change_indices(win_indices.into_iter().map(|v| (v.id, v.zindex)));
                    tx_resp.send(Response::ZIndexChanged(result)).unwrap();
                }
                Request::ListNewWindows => {
                    let new_wins = wm.check_new();
                    tx_resp.send(Response::NewWindows(new_wins)).unwrap();
                }
                Request::RestackWindows => {
                    wm.restack_windows()?;
                    tx_resp.send(Response::RestackComplete).unwrap();
                }
            }
        }

        let diff = Instant::now() - start;
        if diff < min_wait {
            sleep(min_wait - diff);
        }
    }
}
