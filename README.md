# Simple X11 window manager

This window manager delegates all control over to stdin. You communicate with it using JSON messages. It only supports a very limited set of commands, but the code should be easy to hack to add new functionality. 

This window manager is designed around two lists of *mapped* windows: *visible* and *hidden*. For this window manager to appear to be functioning correctly it will be necessary to have one large window that spans the entire screen with a completely black background. This window should be set to visible and should ideally have a zindex of 0. The window manager does not automatically spawn such a window, the user is expected to provide it.

The following are the supported commands along with example input:

1. **ListNewWindows** - returns a list of all new windows that were mapped since the last invocation of this very same command.

   ```json
   "ListNewWindows"
   ```

2. **ListVisibleWindows** - returns the list of window IDs in the visible list.

   ```json
   "ListVisibleWindows"
   ```

3. **ListHiddenWindows** - returns the list of window IDs in the hidden list.

   ```json
   "ListHiddenWindows"
   ```

4. **FocusWindow** - sets a window to have input focus.

   ```json
   {"FocusWindow":123124}
   ```

5. **ChangeVisibility** - move window to either the hidden or visible list. Window in the hidden list are always stacked below the lowest z-indexed window in the visible list. This command has no visual effect until the RestackWindows command in invoked.

   ```json
   {"ChangeVisibility": [
       {"id":123124, "visible":true},
       {"id":123125, "visible":false}
   ]}
   ```

6. **ChangeZIndex** - changes the z-index or priority of windows. Higher valued z-indices are on top of lower valued z-indices, however this only has effect when the window is moved to the visible list. This command has no visual effect until the RestackWindows command is invoked.

   ```json
   {"ChangeZIndex": [
   	{"id":123124, "zindex":2},
       {"id":123125, "zindex":3}
   ]}
   ```

7. **RestackWindows** - Perform the sorting and re-stack of windows across the visible and hidden lists.

   ```json
   "RestackWindows"
   ```

Unmapped windows are automatically removed from the list they were in.

