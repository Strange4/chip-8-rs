use std::sync::Mutex;

pub static INTERVAL_HANDLE: Mutex<Option<i32>> = Mutex::new(None);
pub static RENDER_DEBUGGER: Mutex<bool> = Mutex::new(false);
pub static BREAKPOINTS: Mutex<Vec<usize>> = Mutex::new(Vec::new());
