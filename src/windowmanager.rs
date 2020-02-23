use std::collections::HashMap;

use std::process::exit;

use x11rb::connection::Connection;
use x11rb::generated::xproto::*;
use x11rb::x11_utils::{Event, GenericEvent};
use x11rb::xcb_ffi::XCBConnection;

pub use x11rb::errors::ConnectionErrorOrX11Error;
pub use x11rb::generated::xproto::WINDOW;

#[derive(Clone, Debug)]
pub struct WinInfo {
    pub id: WINDOW,
    pub index: u32,
}

pub struct WindowManager {
    conn: XCBConnection,
    screen_num: usize,
    visible_wins: HashMap<WINDOW, WinInfo>,
    hidden_wins: HashMap<WINDOW, WinInfo>,
    new_wins: Vec<WINDOW>,
}

impl WindowManager {
    pub fn new() -> Result<Self, ConnectionErrorOrX11Error> {
        let (conn, screen_num) = XCBConnection::connect(None)?;

        let mut wm = WindowManager {
            conn: conn,
            screen_num: screen_num,
            visible_wins: HashMap::new(),
            hidden_wins: HashMap::new(),
            new_wins: Vec::new(),
        };

        wm.become_wm()?;
        wm.scan_windows()?;

        Ok(wm)
    }

    pub fn process_events(&mut self) -> Result<(), ConnectionErrorOrX11Error> {
        if let Ok(Some(event)) = self.conn.poll_for_event() {
            self.handle_event(event)
        } else {
            Ok(())
        }
    }

    pub fn change_indices<'a, I>(&mut self, iter: I) -> Vec<WINDOW>
    where
        I: Iterator<Item = (WINDOW, u32)>,
    {
        let mut changed_wins = Vec::new();

        for (id, index) in iter {
            if let Some(v) = self.hidden_wins.get_mut(&id) {
                if v.index != index {
                    v.index = index;
                    changed_wins.push(id);
                }
            } else if let Some(v) = self.visible_wins.get_mut(&id) {
                if v.index != index {
                    v.index = index;
                    changed_wins.push(id);
                }
            }
        }

        changed_wins
    }

    pub fn change_visiblity<'a, I>(&mut self, iter: I) -> Vec<WINDOW>
    where
        I: Iterator<Item = (WINDOW, bool)>,
    {
        let mut changed_wins = Vec::new();

        for (winid, to_visible) in iter {
            if to_visible {
                if let Some(v) = self.hidden_wins.remove(&winid) {
                    self.visible_wins.insert(winid, v);
                    changed_wins.push(winid);
                }
            } else {
                if let Some(v) = self.visible_wins.remove(&winid) {
                    self.hidden_wins.insert(winid, v);
                    changed_wins.push(winid);
                }
            }
        }

        changed_wins
    }

    // restack windows
    pub fn restack_windows(&self) -> Result<(), ConnectionErrorOrX11Error> {
        let mut aux = ConfigureWindowAux::default();

        // sort visible by zindex
        let mut sorted_visible = self.visible_wins.values().collect::<Vec<_>>();
        sorted_visible.sort_unstable_by_key(|v| v.index);

        // push all hidden to bottom
        aux = aux.stack_mode(StackMode::Below);
        for wininfo in self.hidden_wins.values() {
            eprintln!(
                "hidden window {:#x} has been restacked {:?}",
                wininfo.id, aux
            );
            self.conn.configure_window(wininfo.id, &aux)?;
        }

        // stack sorted visible windows
        aux = aux.stack_mode(StackMode::Above);
        for wininfo in sorted_visible {
            eprintln!(
                "visible window {:#x} has been restacked {:?}",
                wininfo.id, aux
            );
            self.conn.configure_window(wininfo.id, &aux)?;
        }

        self.conn.flush();

        Ok(())
    }

    // check new wins are remove them from new window queue
    pub fn check_new(&mut self) -> Vec<WINDOW> {
        self.new_wins.drain(..).collect()
    }

    fn screen_ref(&self) -> &Screen {
        &self.conn.setup().roots[self.screen_num]
    }

    fn become_wm(&self) -> Result<(), ConnectionErrorOrX11Error> {
        let mask = EventMask::SubstructureRedirect
            | EventMask::SubstructureNotify
            | EventMask::EnterWindow;

        let change = ChangeWindowAttributesAux::default().event_mask(mask);

        let root = self.screen_ref().root;

        let error = self.conn.change_window_attributes(root, &change)?.check()?;

        if let Some(error) = error {
            if error.error_code() == ACCESS_ERROR {
                eprintln!("Another WM is already running.");
                exit(1);
            }
            return Err(error.into());
        }

        Ok(())
    }

    fn scan_windows(&mut self) -> Result<(), ConnectionErrorOrX11Error> {
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
            let unmapped: u8 = MapState::Unmapped.into();

            // ignore unmapped windows, or windows with override-redirect set
            if !attr.override_redirect && attr.map_state != unmapped {
                wins.entry(win).or_insert(WinInfo { id: win, index: 0 });
                self.new_wins.push(win);
            }
        }

        Ok(())
    }

    fn handle_configure_request(
        &self,
        event: ConfigureRequestEvent,
    ) -> Result<(), ConnectionErrorOrX11Error> {
        let mut aux = ConfigureWindowAux::default();

        let x: u16 = ConfigWindow::X.into();
        let y: u16 = ConfigWindow::Y.into();
        let w: u16 = ConfigWindow::Width.into();
        let h: u16 = ConfigWindow::Height.into();

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

        aux = aux.stack_mode(StackMode::Below);

        eprintln!("window {:#x} has been reconfigured {:?}", event.window, aux);
        self.conn.configure_window(event.window, &aux)?;

        Ok(())
    }

    fn handle_map_request(
        &mut self,
        event: MapRequestEvent,
    ) -> Result<(), ConnectionErrorOrX11Error> {
        // let geom = self.conn.get_geometry(event.window)?.reply()?;
        let win = event.window;

        // track window
        self.hidden_wins
            .entry(win)
            .or_insert(WinInfo { id: win, index: 0 });

        self.new_wins.push(win);

        self.conn.map_window(win)?;

        Ok(())
    }

    fn handle_unmap_notify(
        &mut self,
        event: UnmapNotifyEvent,
    ) -> Result<(), ConnectionErrorOrX11Error> {
        self.hidden_wins.remove(&event.window);
        self.visible_wins.remove(&event.window);
        eprintln!("window {:#x} unmapped and removed", event.window);
        Ok(())
    }

    // focus follows mouse
    fn handle_enter(&self, event: EnterNotifyEvent) -> Result<(), ConnectionErrorOrX11Error> {
        let window = if let Some(_) = self.visible_wins.get(&event.child) {
            event.child
        } else {
            event.event
        };

        self.conn.set_input_focus(InputFocus::Parent, window, 0)?;
        eprintln!("Window {:#x} got focused", event.child);
        Ok(())
    }

    fn handle_event(&mut self, event: GenericEvent) -> Result<(), ConnectionErrorOrX11Error> {
        // println!("Got event {:?}", event);

        match event.response_type() {
            UNMAP_NOTIFY_EVENT => self.handle_unmap_notify(event.into()),
            CONFIGURE_REQUEST_EVENT => self.handle_configure_request(event.into()),
            MAP_REQUEST_EVENT => self.handle_map_request(event.into()),
            ENTER_NOTIFY_EVENT => self.handle_enter(event.into()),
            _ => Ok(()),
        }
    }
}
