macro_rules! break_on_flag {
    ($atomic:expr, $closure:expr) => {{
        if $atomic.load(std::sync::atomic::Ordering::Relaxed) {
            #[allow(clippy::redundant_closure_call)]
            $closure();
            break;
        }
    }};
}
macro_rules! return_on_flag {
    ($atomic:expr, $closure:expr) => {{
        if $atomic.load(std::sync::atomic::Ordering::Relaxed) {
            #[allow(clippy::redundant_closure_call)]
            $closure();
            return Ok(());
        }
    }};
}
pub(crate) use break_on_flag;
pub(crate) use return_on_flag;
