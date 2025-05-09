use std::sync::{Arc, Mutex};
use warp::Filter;
use x11wmgr::messages::*;
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

    let api = warp::path("api").and(warp::path("windows"));

    let list_new_windows = api
        .and(warp::path("new"))
        .and(warp::get())
        .and(with_wm(wm.clone()))
        .and_then(handle_list_new_windows);

    let list_visible_windows = api
        .and(warp::path("visible"))
        .and(warp::get())
        .and(with_wm(wm.clone()))
        .and_then(handle_list_visible_windows);

    let list_hidden_windows = api
        .and(warp::path("hidden"))
        .and(warp::get())
        .and(with_wm(wm.clone()))
        .and_then(handle_list_hidden_windows);

    let focus_window = api
        .and(warp::path("focus"))
        .and(warp::post())
        .and(with_wm(wm.clone()))
        .and(warp::body::json())
        .and_then(handle_focus_window);

    let change_visibility = api
        .and(warp::path("visibility"))
        .and(warp::post())
        .and(with_wm(wm.clone()))
        .and(warp::body::json())
        .and_then(handle_change_visibility);

    let move_windows = api
        .and(warp::path("move"))
        .and(warp::post())
        .and(with_wm(wm.clone()))
        .and(warp::body::json())
        .and_then(handle_move_windows);

    let resize_windows = api
        .and(warp::path("resize"))
        .and(warp::post())
        .and(with_wm(wm.clone()))
        .and(warp::body::json())
        .and_then(handle_resize_windows);

    let change_zindex = api
        .and(warp::path("zindex"))
        .and(warp::post())
        .and(with_wm(wm.clone()))
        .and(warp::body::json())
        .and_then(handle_change_zindex);

    let commit = api
        .and(warp::path("commit"))
        .and(warp::post())
        .and(with_wm(wm.clone()))
        .and_then(handle_commit);

    let routes = list_new_windows
        .or(list_visible_windows)
        .or(list_hidden_windows)
        .or(focus_window)
        .or(change_visibility)
        .or(move_windows)
        .or(resize_windows)
        .or(change_zindex)
        .or(commit);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

async fn handle_list_new_windows(
    wm: Arc<Mutex<WindowManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut wm = wm.lock().unwrap();
    let new_wins = wm.check_new();
    Ok(warp::reply::json(&Response::NewWindows(new_wins)))
}

async fn handle_list_visible_windows(
    wm: Arc<Mutex<WindowManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let wm = wm.lock().unwrap();
    let wins = wm.get_visible_wins();
    Ok(warp::reply::json(&Response::VisibleWindows(wins)))
}

async fn handle_list_hidden_windows(
    wm: Arc<Mutex<WindowManager>>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let wm = wm.lock().unwrap();
    let wins = wm.get_hidden_wins();
    Ok(warp::reply::json(&Response::HiddenWindows(wins)))
}

async fn handle_focus_window(
    wm: Arc<Mutex<WindowManager>>,
    id: Window,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut wm = wm.lock().unwrap();
    let is_focused = wm.focus_window(id)?;
    Ok(warp::reply::json(&Response::WindowFocused(is_focused)))
}

async fn handle_change_visibility(
    wm: Arc<Mutex<WindowManager>>,
    win_vis: Vec<WinVisbilty>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut wm = wm.lock().unwrap();
    let result = wm.change_visiblity(win_vis.into_iter());
    Ok(warp::reply::json(&Response::VisibiltyChanged(result)))
}

async fn handle_move_windows(
    wm: Arc<Mutex<WindowManager>>,
    windows: Vec<WinMove>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut wm = wm.lock().unwrap();
    wm.move_windows(windows.into_iter())?;
    Ok(warp::reply::json(&Response::MoveComplete))
}

async fn handle_resize_windows(
    wm: Arc<Mutex<WindowManager>>,
    windows: Vec<WinResize>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut wm = wm.lock().unwrap();
    wm.resize_windows(windows.into_iter())?;
    Ok(warp::reply::json(&Response::ResizeComplete))
}

async fn handle_change_zindex(
    wm: Arc<Mutex<WindowManager>>,
    win_indices: Vec<WinZIndex>,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut wm = wm.lock().unwrap();
    let result = wm.change_indices(win_indices.into_iter());
    Ok(warp::reply::json(&Response::ZIndexChanged(result)))
}

async fn handle_commit(wm: Arc<Mutex<WindowManager>>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut wm = wm.lock().unwrap();
    wm.commit()?;
    Ok(warp::reply::json(&Response::CommitComplete))
}

fn with_wm(
    wm: Arc<Mutex<WindowManager>>,
) -> impl Filter<Extract = (Arc<Mutex<WindowManager>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || wm.clone())
}
