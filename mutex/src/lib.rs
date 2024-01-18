//! Minimal implementation of mutex

#![no_std]

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/* -------------------------------------------------------------------------------- */

/// A mutual exclusion primitive, useful for protecting shared data
#[derive(Debug, Default)]
pub struct Mutex<T> {
    /// Data being protected
    data: UnsafeCell<T>,
    /// Lock state of this mutex
    lock: AtomicBool,
    // TODO: poisoned: AtomicBool,
}
unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Sync> Sync for Mutex<T> {}
impl<T> Mutex<T> {
    /// Create a new mutex in an unlocked state ready for use
    pub const fn new(data: T) -> Self {
        let data = UnsafeCell::new(data);
        let lock = AtomicBool::new(false);
        Self { data, lock }
    }

    /// Attempt to acquire this lock
    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        match self
            .lock
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
        {
            Ok(_) => Some(MutexGuard { mutex: self }),
            Err(_) => None,
        }
    }

    /// Acquire this lock, blocking the current thread until it is lockable
    pub fn spin_lock(&self) -> MutexGuard<T> {
        loop {
            if let Some(guard) = self.try_lock() {
                break guard;
            }
        }
    }
}

/* -------------------------------------------------------------------------------- */

/// An RAII implementation of a “scoped lock” of a mutex
#[must_use]
#[derive(Debug)]
pub struct MutexGuard<'a, T> {
    /// Mutex that this guard is locking
    mutex: &'a Mutex<T>,
}
impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.mutex.data.get() }
    }
}
impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.mutex.data.get() }
    }
}
impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.lock.store(false, Ordering::Release);
    }
}

/* -------------------------------------------------------------------------------- */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_lock() {
        let mutex = Mutex::new(());

        {
            let lock_1 = mutex.try_lock();
            assert!(lock_1.is_some());
            let lock_2 = mutex.try_lock();
            assert!(lock_2.is_none());
        }

        let lock_1 = mutex.try_lock();
        assert!(lock_1.is_some());
        let lock_2 = mutex.try_lock();
        assert!(lock_2.is_none());

        drop(lock_1);

        let lock = mutex.try_lock();
        assert!(lock.is_some());
    }
}
