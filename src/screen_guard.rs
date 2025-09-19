use crate::helpers::reset_scr;

pub struct ScreenGuard;

impl Drop for ScreenGuard {
    /// Make sure to call reset_scr even if any fails happen in the main loop
    fn drop(&mut self) {
        if let Err(e) = reset_scr() {
            eprintln!("Error while resetting screen: {e}");
        }
    }
}
