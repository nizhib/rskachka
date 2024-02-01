macro_rules! break_on_flag {
    ($mutex:expr, $message:expr) => {{
        let is_flagged = $mutex.lock().unwrap();
        if *is_flagged {
            log::info!($message);
            break;
        }
    }};
}
macro_rules! return_on_flag {
    ($mutex:expr, $message:expr) => {{
        let is_flagged = $mutex.lock().unwrap();
        if *is_flagged {
            log::info!($message);
            return Ok(());
        }
    }};
}
pub(crate) use break_on_flag;
pub(crate) use return_on_flag;
