use std::sync::mpsc::channel;

use std::process::exit;

use x11wmgr::*;

fn main() {
    let name = std::env::args().nth(0).unwrap_or_default();
    if let Err(err) = run() {
        if name.is_empty() {
            eprintln!("{}", err);
        } else {
            eprintln!("{}: {}", name, err);
        }

        exit(1);
    }
}

fn run() -> Result<(), Error> {
    let mut wm = WindowManager::new()?;

    let waker = wm.create_waker()?;

    let (tx_req, rx_req) = channel::<Request>();
    let tx_resp = create_cli(waker, tx_req);

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
        Request::ChangeVisibility(win_vis) => {
            let result = wm.change_visiblity(win_vis.into_iter().map(|v| (v.id, v.visible)));
            Response::VisibiltyChanged(result)
        }
        Request::ChangeZIndex(win_indices) => {
            let result = wm.change_indices(win_indices.into_iter().map(|v| (v.id, v.zindex)));
            Response::ZIndexChanged(result)
        }
        Request::ListNewWindows => {
            let new_wins = wm.check_new();
            Response::NewWindows(new_wins)
        }
        Request::ListVisibleWindows => {
            let wins = wm
                .get_visible_wins()
                .map(|(id, zindex)| WinZIndex { id, zindex })
                .collect();
            Response::VisibleWindows(wins)
        }
        Request::ListHiddenWindows => {
            let wins = wm
                .get_hidden_wins()
                .map(|(id, zindex)| WinZIndex { id, zindex })
                .collect();
            Response::HiddenWindows(wins)
        }
        Request::RestackWindows => {
            wm.restack_windows()?;
            Response::RestackComplete
        }
        Request::FocusWindow(id) => {
            let is_focused = wm.focus_window(id)?;
            Response::WindowFocused(is_focused)
        }
        Request::ResizeWindows(windows) => {
            wm.resize_windows(windows.into_iter())?;
            Response::ResizeComplete
        }
        Request::MoveWindows(windows) => {
            wm.move_windows(windows.into_iter())?;
            Response::MoveComplete
        }
        Request::MoveResizeWindows(windows) => {
            wm.move_resize_windows(windows.into_iter())?;
            Response::MoveResizeComplete
        }
    };

    Ok(resp)
}
