macro_rules! break_on_flag {
    ($atomic:expr, $message:expr) => {{
        if $atomic.load(std::sync::atomic::Ordering::Relaxed) {
            log::info!($message);
            break;
        }
    }};
}
macro_rules! return_on_flag {
    ($atomic:expr, $message:expr) => {{
        if $atomic.load(std::sync::atomic::Ordering::Relaxed) {
            log::info!($message);
            return Ok(());
        }
    }};
}
pub(crate) use break_on_flag;
pub(crate) use return_on_flag;
