use std::sync::mpsc::channel;

use x11wmgr::*;

fn main() -> Result<(), Error> {
    let mut wm = WindowManager::new()?;

    let waker = wm.create_waker()?;

    let (tx_req, rx_req) = channel::<Request>();
    let tx_resp = create_cli(waker, tx_req)?;

    loop {
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
                Request::ListVisibleWindows => {
                    let wins = wm
                        .get_visible_wins()
                        .into_iter()
                        .map(|(id, indx)| WinZIndex {
                            id: id,
                            zindex: indx,
                        })
                        .collect();
                    tx_resp.send(Response::VisibleWindows(wins)).unwrap();
                }
                Request::ListHiddenWindows => {
                    let wins = wm
                        .get_hidden_wins()
                        .into_iter()
                        .map(|(id, indx)| WinZIndex {
                            id: id,
                            zindex: indx,
                        })
                        .collect();
                    tx_resp.send(Response::HiddenWindows(wins)).unwrap();
                }
                Request::RestackWindows => {
                    wm.restack_windows()?;
                    tx_resp.send(Response::RestackComplete).unwrap();
                }
            }
        }
    }
}
