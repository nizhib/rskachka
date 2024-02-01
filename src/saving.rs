use std::sync::{Condvar, Mutex};

/// A semaphore used for synchronization during saving operations.
pub struct SavingSemaphore {
    count: Mutex<u32>,
    cvar: Condvar,
}

impl SavingSemaphore {
    /// Creates a new `SavingSemaphore` instance.
    pub fn new() -> Self {
        SavingSemaphore {
            count: Mutex::new(0),
            cvar: Condvar::new(),
        }
    }

    /// Increments the semaphore count.
    pub fn increment(&self) {
        let mut count = self.count.lock().unwrap();
        *count += 1;
    }

    /// Decrements the semaphore count and notifies all waiting threads if the count reaches zero.
    pub fn decrement(&self) {
        let mut count = self.count.lock().unwrap();
        *count -= 1;
        if *count == 0 {
            self.cvar.notify_all();
        }
    }

    /// Waits until the semaphore count reaches zero.
    pub fn wait(&self) {
        let mut count = self.count.lock().unwrap();
        while *count > 0 {
            count = self.cvar.wait(count).unwrap();
        }
    }
}
