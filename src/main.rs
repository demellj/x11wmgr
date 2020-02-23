use std::sync::mpsc::channel;

use x11wmgr::*;

fn main() -> Result<(), ConnectionErrorOrX11Error> {
    let mut wm = WindowManager::new()?;

    let (tx_req, rx_req) = channel::<Request>();
    let tx_resp = create_cli(tx_req);

    loop {
        wm.process_events()?;

        if let Ok(req) = rx_req.try_recv() {
            match req {
                Request::ChangeVisiblity(win_vis) => {
                    let result = wm.change_visiblity(win_vis.into_iter().map(|v| (v.id, v.visible)));
                    tx_resp.send(Response::VisibiltyChanged(result));
                }
                Request::ChangeZIndex(win_indices) => {
                    let result = wm.change_indices(win_indices.into_iter().map(|v| (v.id, v.zindex)));
                    tx_resp.send(Response::ZIndexChanged(result));
                }
                Request::ListNewWindow => {
                    let new_wins = wm.check_new();
                    tx_resp.send(Response::NewWindows(new_wins));
                }
                Request::RestackWindows => {
                    wm.restack_windows()?;
                    tx_resp.send(Response::RestackComplete);
                }
            }
        }
    }
}
