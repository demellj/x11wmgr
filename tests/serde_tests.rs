use serde_json;
use x11wmgr::{WindowInfo, Request, Response, WinMove, WinResize, WinVisbilty, WinZIndex};

#[test]
fn test_request_move_windows_serialization() {
    let request = Request::MoveWindows(vec![
        WinMove {
            id: 1,
            x: 100,
            y: 200,
        },
        WinMove {
            id: 2,
            x: -50,
            y: -75,
        },
    ]);

    let serialized = serde_json::to_string(&request).unwrap();
    let expected = r#"{"MoveWindows":[{"id":1,"x":100,"y":200},{"id":2,"x":-50,"y":-75}]}"#;
    assert_eq!(serialized, expected);

    let deserialized: Request = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, request);
}

#[test]
fn test_request_resize_windows_serialization() {
    let request = Request::ResizeWindows(vec![
        WinResize {
            id: 1,
            width: 800,
            height: 600,
        },
        WinResize {
            id: 2,
            width: 1024,
            height: 768,
        },
    ]);

    let serialized = serde_json::to_string(&request).unwrap();
    let expected = r#"{"ResizeWindows":[{"id":1,"width":800,"height":600},{"id":2,"width":1024,"height":768}]}"#;
    assert_eq!(serialized, expected);

    let deserialized: Request = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, request);
}

#[test]
fn test_request_change_visibility_serialization() {
    let request = Request::ChangeVisibility(vec![
        WinVisbilty {
            id: 1,
            visible: true,
        },
        WinVisbilty {
            id: 2,
            visible: false,
        },
    ]);

    let serialized = serde_json::to_string(&request).unwrap();
    let expected = r#"{"ChangeVisibility":[{"id":1,"visible":true},{"id":2,"visible":false}]}"#;
    assert_eq!(serialized, expected);

    let deserialized: Request = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, request);
}

#[test]
fn test_request_change_zindex_serialization() {
    let request = Request::ChangeZIndex(vec![
        WinZIndex {
            id: 1,
            zindex: 10,
        },
        WinZIndex {
            id: 2,
            zindex: 20,
        },
    ]);

    let serialized = serde_json::to_string(&request).unwrap();
    let expected = r#"{"ChangeZIndex":[{"id":1,"zindex":10},{"id":2,"zindex":20}]}"#;
    assert_eq!(serialized, expected);

    let deserialized: Request = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, request);
}

#[test]
fn test_request_list_new_windows_serialization() {
    let request = Request::ListNewWindows;

    let serialized = serde_json::to_string(&request).unwrap();
    let expected = r#""ListNewWindows""#;
    assert_eq!(serialized, expected);

    let deserialized: Request = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, request);
}

#[test]
fn test_request_commit_serialization() {
    let request = Request::Commit;

    let serialized = serde_json::to_string(&request).unwrap();
    let expected = r#""Commit""#;
    assert_eq!(serialized, expected);

    let deserialized: Request = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, request);
}

#[test]
fn test_response_new_windows_serialization() {
    let response = Response::NewWindows(vec![
        WindowInfo {
            id: 1,
            x: 100,
            y: 200,
            width: 800,
            height: 600,
        },
        WindowInfo {
            id: 2,
            x: -50,
            y: -75,
            width: 1024,
            height: 768,
        },
    ]);

    let serialized = serde_json::to_string(&response).unwrap();
    let expected = r#"{"NewWindows":[{"id":1,"x":100,"y":200,"width":800,"height":600},{"id":2,"x":-50,"y":-75,"width":1024,"height":768}]}"#;
    assert_eq!(serialized, expected);

    let deserialized: Response = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, response);
}

#[test]
fn test_response_commit_complete_serialization() {
    let response = Response::CommitComplete;

    let serialized = serde_json::to_string(&response).unwrap();
    let expected = r#""CommitComplete""#;
    assert_eq!(serialized, expected);

    let deserialized: Response = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, response);
}

#[test]
fn test_response_move_complete_serialization() {
    let response = Response::MoveComplete;

    let serialized = serde_json::to_string(&response).unwrap();
    let expected = r#""MoveComplete""#;
    assert_eq!(serialized, expected);

    let deserialized: Response = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, response);
}

#[test]
fn test_response_resize_complete_serialization() {
    let response = Response::ResizeComplete;

    let serialized = serde_json::to_string(&response).unwrap();
    let expected = r#""ResizeComplete""#;
    assert_eq!(serialized, expected);

    let deserialized: Response = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, response);
}

#[test]
fn test_response_window_focused_serialization() {
    let response = Response::WindowFocused(true);

    let serialized = serde_json::to_string(&response).unwrap();
    let expected = r#"{"WindowFocused":true}"#;
    assert_eq!(serialized, expected);

    let deserialized: Response = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, response);
}
