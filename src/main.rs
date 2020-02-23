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
            let resp = handle_request(&mut wm, req)?;
            tx_resp.send(resp)?;
        }
    }
}

fn handle_request(wm: &mut WindowManager, req: Request) -> Result<Response, Error> {
    let resp = match req {
        Request::ChangeVisiblity(win_vis) => {
            let result =
                wm.change_visiblity(win_vis.into_iter().map(|v| (v.id, v.visible)));
            Response::VisibiltyChanged(result)
        }
        Request::ChangeZIndex(win_indices) => {
            let result =
                wm.change_indices(win_indices.into_iter().map(|v| (v.id, v.zindex)));
            Response::ZIndexChanged(result)
        }
        Request::ListNewWindows => {
            let new_wins = wm.check_new();
            Response::NewWindows(new_wins)
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
            Response::VisibleWindows(wins)
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
            Response::HiddenWindows(wins)
        }
        Request::RestackWindows => {
            wm.restack_windows()?;
            Response::RestackComplete
        }
    };

    Ok(resp)
}
