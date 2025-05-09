use std::sync::{Arc, Mutex};
use warp::Filter;
use x11wmgr::messages::{Request, Response};
use x11wmgr::*;

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Error> {
    let wm = Arc::new(Mutex::new(WindowManager::new()?));

    let api = warp::path("api")
        .and(with_wm(wm.clone()))
        .and(warp::body::json())
        .and_then(handle_request);

    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

async fn handle_request(
    wm: Arc<Mutex<WindowManager>>,
    req: Request,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut wm = wm.lock().unwrap();
    let resp = match req {
        Request::ChangeVisibility(win_vis) => {
            let result = wm.change_visiblity(win_vis.into_iter());
            Response::VisibiltyChanged(result)
        }
        Request::ChangeZIndex(win_indices) => {
            let result = wm.change_indices(win_indices.into_iter());
            Response::ZIndexChanged(result)
        }
        Request::ListNewWindows => {
            let new_wins = wm.check_new();
            Response::NewWindows(new_wins)
        }
        Request::ListVisibleWindows => {
            let wins = wm.get_visible_wins();
            Response::VisibleWindows(wins)
        }
        Request::ListHiddenWindows => {
            let wins = wm.get_hidden_wins();
            Response::HiddenWindows(wins)
        }
        Request::Commit => {
            wm.commit()?;
            Response::CommitComplete
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
    };

    Ok(warp::reply::json(&resp))
}

fn with_wm(
    wm: Arc<Mutex<WindowManager>>,
) -> impl Filter<Extract = (Arc<Mutex<WindowManager>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || wm.clone())
}
