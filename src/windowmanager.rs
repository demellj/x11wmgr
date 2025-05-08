use std::cmp;
use std::collections::HashMap;
use std::time::Instant;

use std::sync::Arc;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::rust_connection::RustConnection;
use x11rb::x11_utils::TryParse;
use x11rb::COPY_DEPTH_FROM_PARENT;

pub use x11rb::protocol::xproto::Window;

pub type ZIndexType = u32;

use crate::error::*;
use crate::{WinMove, WinResize};

const PENDING_INPUT_ATOM_NAME: &'static str = "__WMGR_PENDING_INPUT";

#[derive(Clone, Debug)]
struct WinInfo {
    id: Window,
    index: ZIndexType,

    // the time the window was mapped/discovered
    discovery_time: Instant,

    // last time window zindex or visibilty was updated
    last_update_time: Instant,
}

pub struct Waker {
    conn: Arc<RustConnection>,
    win: Window,
    event: ClientMessageEvent,
}

pub struct WindowManager {
    conn: Arc<RustConnection>,
    windows_loc: HashMap<Window, (i32, i32)>,
    windows_size: HashMap<Window, (u32, u32)>,
    screen_num: usize,

    // window that spans the entire screen and has a black background.
    virtual_root_win: Window,

    // windows that are currently in the visible stack
    visible_wins: HashMap<Window, WinInfo>,

    // windows that are currently in the hidden stack
    hidden_wins: HashMap<Window, WinInfo>,

    // the last time new windows were queried
    last_discovery_time: Instant,

    // pending input atom
    pending_input_atom: Atom,
}

impl Waker {
    // wake up wm thread, notifying it of pending input
    pub fn wake(&self) -> Result<(), Error> {
        let cookie =
            self.conn
                .send_event(false, self.win, EventMask::SUBSTRUCTURE_NOTIFY, &self.event)?;

        cookie.check()?;

        Ok(())
    }
}

impl WindowManager {
    pub fn new() -> Result<Self, Error> {
        let (conn, screen_num) = RustConnection::connect(None)?;

        let pending_input_atom = intern_atom(&conn, false, PENDING_INPUT_ATOM_NAME.as_bytes())?
            .reply()?
            .atom;

        let screen = &conn.setup().roots[screen_num];

        let wid = conn.generate_id()?;

        conn.create_window(
            COPY_DEPTH_FROM_PARENT,
            wid,
            screen.root,
            0,
            0,
            screen.width_in_pixels,
            screen.height_in_pixels,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().background_pixel(screen.black_pixel),
        )?;

        conn.map_window(wid)?;
        conn.flush()?;

        let mut wm = WindowManager {
            windows_loc: HashMap::new(),
            windows_size: HashMap::new(),
            conn: Arc::new(conn),
            screen_num,
            virtual_root_win: wid,
            visible_wins: HashMap::new(),
            hidden_wins: HashMap::new(),
            last_discovery_time: Instant::now(),
            pending_input_atom,
        };

        wm.become_wm()?;
        wm.scan_windows()?;

        Ok(wm)
    }

    pub fn process_events(&mut self) -> Result<(), Error> {
        while let Ok(event) = self.conn.wait_for_event() {
            if !self.handle_event(event)? {
                break;
            }
        }
        Ok(())
    }

    pub fn create_waker(&self) -> Result<Waker, Error> {
        let atom = self.pending_input_atom;

        let mut data = [0; 20];
        data[..4].copy_from_slice(&atom.to_ne_bytes());

        let (data, _): (ClientMessageData, _) = TryParse::try_parse(&data)?;

        let win = self.screen_ref().root;

        let event = ClientMessageEvent {
            response_type: CLIENT_MESSAGE_EVENT,
            format: 32,
            sequence: 0,
            window: win,
            type_: atom,
            data,
        };

        Ok(Waker {
            conn: self.conn.clone(),
            win,
            event,
        })
    }

    pub fn change_indices<I>(&mut self, iter: I) -> Vec<Window>
    where
        I: Iterator<Item = (Window, ZIndexType)>,
    {
        let mut changed_wins = Vec::new();

        for (id, index) in iter {
            if let Some(v) = self.hidden_wins.get_mut(&id) {
                if v.index != index {
                    v.index = index;
                    v.last_update_time = Instant::now();
                    changed_wins.push(id);
                }
            } else if let Some(v) = self.visible_wins.get_mut(&id) {
                if v.index != index {
                    v.index = index;
                    v.last_update_time = Instant::now();
                    changed_wins.push(id);
                }
            }
        }

        changed_wins
    }

    pub fn change_visiblity<I>(&mut self, iter: I) -> Vec<Window>
    where
        I: Iterator<Item = (Window, bool)>,
    {
        let mut changed_wins = Vec::new();

        for (winid, to_visible) in iter {
            if to_visible {
                if let Some(mut v) = self.hidden_wins.remove(&winid) {
                    v.last_update_time = Instant::now();
                    self.visible_wins.insert(winid, v);
                    changed_wins.push(winid);
                }
            } else {
                if let Some(mut v) = self.visible_wins.remove(&winid) {
                    v.last_update_time = Instant::now();
                    self.hidden_wins.insert(winid, v);
                    changed_wins.push(winid);
                }
            }
        }

        changed_wins
    }

    // synchronous
    pub fn focus_window(&self, id: Window) -> Result<bool, Error> {
        if self.visible_wins.contains_key(&id) {
            let cookie = self
                .conn
                .set_input_focus(InputFocus::PARENT, id, Time::CURRENT_TIME)?;

            cookie.check()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    // resize multiple windows (deferred)
    pub fn resize_windows<I>(&mut self, iter: I) -> Result<(), Error>
    where
        I: Iterator<Item = WinResize>,
    {
        for WinResize { id, width, height } in iter {
            self.windows_size.insert(id, (width, height));
        }
        Ok(())
    }

    // move multiple windows (deferred)
    pub fn move_windows<I>(&mut self, iter: I) -> Result<(), Error>
    where
        I: Iterator<Item = WinMove>,
    {
        for WinMove { id, x, y } in iter {
            self.windows_loc.insert(id, (x, y));
        }
        Ok(())
    }

    // restack windows (synchronous)
    pub fn restack_windows(&mut self) -> Result<(), Error> {
        let mut aux = ConfigureWindowAux::default();

        // sort visible by zindex
        let mut sorted_visible = self.visible_wins.values().collect::<Vec<_>>();
        sorted_visible.sort_unstable_by_key(|v| v.index);

        // push all hidden to bottom
        aux = aux.stack_mode(StackMode::BELOW);
        for wininfo in self.hidden_wins.values() {
            self.conn.configure_window(wininfo.id, &aux)?;
        }

        // push virtual root window
        aux = aux.stack_mode(StackMode::ABOVE);
        self.conn.configure_window(self.virtual_root_win, &aux)?;

        // stack sorted visible windows above it
        for wininfo in sorted_visible {
            self.conn.configure_window(wininfo.id, &aux)?;
        }

        // Apply pending move operations
        for (id, (x, y)) in self.windows_loc.drain() {
            let aux = ConfigureWindowAux::default().x(x).y(y);
            self.conn.configure_window(id, &aux)?;
        }

        // Apply pending resize operations
        for (id, (width, height)) in self.windows_size.drain() {
            let aux = ConfigureWindowAux::default().width(width).height(height);
            self.conn.configure_window(id, &aux)?;
        }

        self.conn.flush()?;

        Ok(())
    }

    // check for newly discovered/mapped windows, sorted by recency,
    // with most recent windows frist
    pub fn check_new(&mut self) -> Vec<Window> {
        // new windows only go into hidden_wins
        let mut new_wins = self
            .hidden_wins
            .values()
            .filter(|winfo| winfo.discovery_time >= self.last_discovery_time)
            .map(|winfo| winfo.id)
            .collect::<Vec<_>>();

        new_wins.sort_unstable_by_key(|w| {
            cmp::Reverse(
                self.hidden_wins
                    .get(w)
                    .and_then(|w| Some(w.index))
                    .unwrap_or(0),
            )
        });
        self.last_discovery_time = Instant::now();
        new_wins
    }

    pub fn get_visible_wins(&self) -> impl Iterator<Item = (Window, ZIndexType)> + '_ {
        self.visible_wins.values().map(|v| (v.id, v.index))
    }

    pub fn get_hidden_wins(&self) -> impl Iterator<Item = (Window, ZIndexType)> + '_ {
        self.hidden_wins.values().map(|v| (v.id, v.index))
    }

    fn screen_ref(&self) -> &Screen {
        &self.conn.setup().roots[self.screen_num]
    }

    fn become_wm(&self) -> Result<(), Error> {
        let mask = EventMask::SUBSTRUCTURE_REDIRECT
            | EventMask::SUBSTRUCTURE_NOTIFY
            | EventMask::ENTER_WINDOW;

        let change = ChangeWindowAttributesAux::default().event_mask(mask);

        let root = self.screen_ref().root;

        self.conn.change_window_attributes(root, &change)?.check()?;

        Ok(())
    }

    fn scan_windows(&mut self) -> Result<(), Error> {
        let tree_reply = self.conn.query_tree(self.screen_ref().root)?.reply()?;

        // For each window, request its attributes and geometry *now*
        let mut cookies = Vec::with_capacity(tree_reply.children.len());
        for win in tree_reply.children {
            let attr = self.conn.get_window_attributes(win)?;
            cookies.push((win, attr));
        }

        // Get the replies and manage windows
        let mut resp = Vec::with_capacity(cookies.len());
        for (win, attr) in cookies {
            if let Ok(attr) = attr.reply() {
                resp.push((win, attr));
            }
        }

        let wins = &mut self.hidden_wins;

        let vroot_win = self.virtual_root_win;

        for (win, attr) in resp {
            // ignore virtual_root_win or unmapped windows or windows with override-redirect set
            if win != vroot_win && !attr.override_redirect && attr.map_state != MapState::UNMAPPED {
                wins.entry(win).or_insert(WinInfo {
                    id: win,
                    index: 0,
                    discovery_time: Instant::now(),
                    last_update_time: Instant::now(),
                });
            }
        }

        Ok(())
    }

    fn handle_configure_request(&mut self, event: ConfigureRequestEvent) -> Result<(), Error> {
        let mut aux = ConfigureWindowAux::default();

        let x: u16 = ConfigWindow::X.into();
        let y: u16 = ConfigWindow::Y.into();
        let w: u16 = ConfigWindow::WIDTH.into();
        let h: u16 = ConfigWindow::HEIGHT.into();

        let event_mask: u16 = event.value_mask.into();

        if event_mask & x != 0 {
            aux = aux.x(i32::from(event.x));
        }
        if event_mask & y != 0 {
            aux = aux.y(i32::from(event.y));
        }
        if event_mask & w != 0 {
            aux = aux.width(u32::from(event.width));
        }
        if event_mask & h != 0 {
            aux = aux.height(u32::from(event.height));
        }

        aux = aux.stack_mode(StackMode::BELOW);

        if event_mask & x != 0 && event_mask & y != 0 {
            self.windows_loc
                .insert(event.window, (i32::from(event.x), i32::from(event.y)));
        }

        if event_mask & w != 0 && event_mask & h != 0 {
            self.windows_size.insert(
                event.window,
                (u32::from(event.width), u32::from(event.height)),
            );
        }

        self.conn.configure_window(event.window, &aux)?;

        Ok(())
    }

    fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<(), Error> {
        let win = event.window;

        // track window
        self.hidden_wins.entry(win).or_insert(WinInfo {
            id: win,
            index: 0,
            discovery_time: Instant::now(),
            last_update_time: Instant::now(),
        });

        self.conn.map_window(win)?;

        Ok(())
    }

    fn handle_unmap_notify(&mut self, event: UnmapNotifyEvent) -> Result<(), Error> {
        self.hidden_wins.remove(&event.window);
        self.visible_wins.remove(&event.window);
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<bool, Error> {
        match event {
            Event::UnmapNotify(une) => {
                self.handle_unmap_notify(une)?;
            }
            Event::ConfigureRequest(cre) => {
                self.handle_configure_request(cre)?;
            }
            Event::MapRequest(mre) => {
                self.handle_map_request(mre)?;
            }
            Event::ClientMessage(msg_event) => {
                if msg_event.type_ == self.pending_input_atom {
                    return Ok(false);
                }
            }
            _ => (),
        }

        Ok(true)
    }
}
