# Simple X11 Window Manager

This window manager delegates all control over to stdin. You communicate with it using JSON messages. It supports a set of commands for managing windows, but all changes require invoking the **Commit** command to take effect. The code is designed to be easy to hack for adding new functionality.

This window manager is designed around two lists of *mapped* windows: *visible* and *hidden*. Windows are never unmapped, but simply moved between these two lists. When the windows are restacked, the hidden windows are covered by a large window spanning the whole screen having a black background. The visible windows are stacked above it according to their z-index.

Additionally, the project includes an optional web service that exposes the window manager's functionality via HTTP APIs, making it accessible over the network.

The following are the supported commands. Note that all commands that modify window state (e.g., visibility, position, size, or z-index) require invoking the **Commit** command to apply the changes. Example inputs are provided for each command. For users of the web service, these commands are also available as HTTP endpoints (details below).

1. **ListNewWindows** - returns a list of all new windows that were mapped since the last invocation of this very same command. Each window in the response includes its ID, position (`x`, `y`), and dimensions (`width`, `height`). New windows are automatically added to the hidden list.

   ```json
   "ListNewWindows"
   ```

2. **ListVisibleWindows** - returns a list of all visible windows. Each window in the response includes its ID, position (`x`, `y`), and dimensions (`width`, `height`).

   ```json
   "ListVisibleWindows"
   ```

3. **ListHiddenWindows** - returns a list of all hidden windows. Each window in the response includes its ID, position (`x`, `y`), and dimensions (`width`, `height`).

   ```json
   "ListHiddenWindows"
   ```

4. **FocusWindow** - sets a window to have input focus.

   ```json
   {"FocusWindow":123124}
   ```

5. **ChangeVisibility** - move window to either the hidden or visible list. Window in the hidden list are always stacked below the lowest z-indexed window in the visible list. This command has no visual effect until the Commit command is invoked.

   ```json
   {"ChangeVisibility": [
       {"id":123124, "visible":true},
       {"id":123125, "visible":false}
   ]}
   ```

6. **MoveWindows** - update the position of one or more windows. This command has no visual effect until the Commit command is invoked.

   ```json
   {"MoveWindows": [
       {"id":123124, "x":100, "y":200},
       {"id":123125, "x":-300, "y":-400}
   ]}
   ```

7. **ResizeWindows** - update the size of one or more windows. This command has no visual effect until the Commit command is invoked.

   ```json
   {"ResizeWindows": [
       {"id":123124, "width":800, "height":600},
       {"id":123125, "width":1024, "height":768}
   ]}
   ```

6. **ChangeZIndex** - changes the z-index or priority of windows. Higher valued z-indices are on top of lower valued z-indices, however this only has effect when the window is moved to the visible list. This command has no visual effect until the Commit command is invoked.

   ```json
   {"ChangeZIndex": [
       {"id":123124, "zindex":2},
       {"id":123125, "zindex":3}
   ]}
   ```

8. **Commit** - Apply all pending changes (e.g., moves, resizes, visibility, and z-index updates) and perform the sorting and re-stack of windows across the visible and hidden lists.

   ```json
   "Commit"
   ```

Unmapped windows are automatically removed from the list they were in. Remember to invoke the **Commit** command after issuing any of the following commands to see the changes take effect: **ChangeVisibility**, **ChangeZIndex**, **MoveWindows**, or **ResizeWindows**.

## Web Service (Optional)

The project includes an optional web service that exposes the window manager's functionality via HTTP APIs. To enable this feature, use the `websrvc` feature when building the project.

### Example Endpoints

- `GET /api/windows/new`: List new windows.
- `GET /api/windows/visible`: List visible windows.
- `GET /api/windows/hidden`: List hidden windows.
- `POST /api/windows/focus`: Focus a window (requires a JSON body with the window ID).
- `POST /api/windows/visibility`: Change window visibility (requires a JSON body).
- `POST /api/windows/move`: Move windows (requires a JSON body).
- `POST /api/windows/resize`: Resize windows (requires a JSON body).
- `POST /api/windows/zindex`: Change window z-index (requires a JSON body).
- `POST /api/windows/commit`: Commit changes.

### Running the Web Service

To run the web service:

```bash
cargo run --bin websrvc --features websrvc
```

The web service will start on `http://127.0.0.1:3030`.
