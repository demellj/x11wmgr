use std::cmp;
use std::collections::HashMap;
use std::time::Instant;

use std::sync::Arc;

use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::Event;
use x11rb::x11_utils::TryParse;
use x11rb::rust_connection::RustConnection;

pub use x11rb::protocol::xproto::Window;

pub type ZIndexType = u32;

use crate::error::*;

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
    screen_num: usize,

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
        let cookie = self.conn.send_event(
            false,
            self.win,
            EventMask::SUBSTRUCTURE_NOTIFY,
            &self.event,
        )?;

        cookie.check()?;

        Ok(())
    }
}

impl WindowManager {
    pub fn new() -> Result<Self, Error> {
        let (conn, screen_num) = RustConnection::connect(None)?;

        let pending_input_atom =
            intern_atom(&conn, false, PENDING_INPUT_ATOM_NAME.as_bytes())?.reply()?.atom;

        let mut wm = WindowManager {
            conn: Arc::new(conn),
            screen_num,
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

    pub fn change_indices<'a, I>(&mut self, iter: I) -> Vec<Window>
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

    pub fn change_visiblity<'a, I>(&mut self, iter: I) -> Vec<Window>
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
            let cookie = self.conn.set_input_focus(InputFocus::PARENT, id, Time::CURRENT_TIME)?;

            cookie.check()?;

            Ok(true)
        } else {
            Ok(false)
        }
    }

    // restack windows (synchronous)
    pub fn restack_windows(&self) -> Result<(), Error> {
        let mut aux = ConfigureWindowAux::default();

        // sort visible by zindex
        let mut sorted_visible = self.visible_wins.values().collect::<Vec<_>>();
        sorted_visible.sort_unstable_by_key(|v| v.index);

        // push all hidden to bottom
        aux = aux.stack_mode(StackMode::BELOW);
        for wininfo in self.hidden_wins.values() {
            eprintln!(
                "hidden window {:#x} has been restacked {:?}",
                wininfo.id, aux
            );
            self.conn.configure_window(wininfo.id, &aux)?;
        }

        // stack sorted visible windows
        aux = aux.stack_mode(StackMode::ABOVE);
        for wininfo in sorted_visible {
            eprintln!(
                "visible window {:#x} has been restacked {:?}",
                wininfo.id, aux
            );
            self.conn.configure_window(wininfo.id, &aux)?;
        }

        self.conn.flush()?;

        Ok(())
    }

    // check for newly discovered/mapped windows, sorted by recency,
    // with most recent windows frist
    pub fn check_new(&mut self) -> Vec<Window> {
        let mut new_wins = Vec::new();
        // new windows only go into hidden_wins
        for wininfo in self.hidden_wins.values() {
            if wininfo.discovery_time >= self.last_discovery_time {
                new_wins.push(wininfo.id);
            }
        }
        new_wins.sort_by_cached_key(|w| {
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

    pub fn get_visible_wins(&self) -> Vec<(Window, ZIndexType)> {
        self.visible_wins
            .values()
            .map(|v| (v.id, v.index))
            .collect()
    }

    pub fn get_hidden_wins(&self) -> Vec<(Window, ZIndexType)> {
        self.hidden_wins.values().map(|v| (v.id, v.index)).collect()
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

        for (win, attr) in resp {
            // ignore unmapped windows, or windows with override-redirect set
            if !attr.override_redirect && attr.map_state != MapState::UNMAPPED {
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

    fn handle_configure_request(&self, event: ConfigureRequestEvent) -> Result<(), Error> {
        let mut aux = ConfigureWindowAux::default();

        let x: u16 = ConfigWindow::X.into();
        let y: u16 = ConfigWindow::Y.into();
        let w: u16 = ConfigWindow::WIDTH.into();
        let h: u16 = ConfigWindow::HEIGHT.into();

        if event.value_mask & x != 0 {
            aux = aux.x(i32::from(event.x));
        }
        if event.value_mask & y != 0 {
            aux = aux.y(i32::from(event.y));
        }
        if event.value_mask & w != 0 {
            aux = aux.width(u32::from(event.width));
        }
        if event.value_mask & h != 0 {
            aux = aux.height(u32::from(event.height));
        }

        aux = aux.stack_mode(StackMode::BELOW);

        eprintln!("window {:#x} has been reconfigured {:?}", event.window, aux);
        self.conn.configure_window(event.window, &aux)?;

        Ok(())
    }

    fn handle_map_request(&mut self, event: MapRequestEvent) -> Result<(), Error> {
        // let geom = self.conn.get_geometry(event.window)?.reply()?;
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
        eprintln!("window {:#x} unmapped and removed", event.window);
        Ok(())
    }

    fn handle_event(&mut self, event: Event) -> Result<bool, Error> {
        // eprintln!("Got event {:?}", event);

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
