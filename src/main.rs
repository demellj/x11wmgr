use x11rb::errors::ConnectionErrorOrX11Error;

use x11wmgr::WindowManager;

fn main() -> Result<(), ConnectionErrorOrX11Error> {
    let mut wm = WindowManager::new()?;

    loop {
        for win in wm.check_new() {
            eprintln!("{:x} detected", win);
        }

        wm.restack_windows()?;

        wm.process_events()?;
    }
}
