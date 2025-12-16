use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct FreezeLock<T> {
    data: UnsafeCell<T>,
    frozen: AtomicBool,
    lock: RwLock<()>,
}

impl<T> FreezeLock<T> {
    #[inline]
    pub const fn new(data: T) -> Self {
        Self::with(data, false)
    }

    #[inline]
    pub const fn frozen(data: T) -> Self {
        Self::with(data, true)
    }

    #[inline]
    pub const fn with(data: T, frozen: bool) -> Self {
        Self {
            data: UnsafeCell::new(data),
            frozen: AtomicBool::new(frozen),
            lock: RwLock::new(()),
        }
    }

    #[inline]
    pub fn is_frozen(&self) -> bool {
        self.frozen.load(Ordering::Acquire)
    }

    pub fn read(&self) -> FreezeReadGuard<'_, T> {
        let _lock_guard = self.lock.read().unwrap();
        FreezeReadGuard {
            _lock_guard,
            data: unsafe { NonNull::new_unchecked(self.data.get()) },
        }
    }

    pub fn write(&self) -> Option<FreezeWriteGuard<'_, T>> {
        let _lock_guard = self.lock.write().unwrap();

        if self.frozen.load(Ordering::Acquire) {
            None
        } else {
            Some(
                FreezeWriteGuard {
                    _lock_guard,
                    data: unsafe { NonNull::new_unchecked(self.data.get()) },
                    frozen: &self.frozen,
                    marker: PhantomData,
                }
            )
        }
    }

    pub fn freeze(&self) -> &T {
        if !self.is_frozen() {
            let _lock = self.lock.write();
            self.frozen.store(true, Ordering::Release);
        }

        unsafe { &*self.data.get() }
    }

    pub fn get(&self) -> Option<&T> {
        if self.is_frozen() {
            Some(unsafe { &*self.data.get() })
        } else {
            None
        }
    }
}

impl<T: Clone> Clone for FreezeLock<T> {
    fn clone(&self) -> Self {
        Self::with(self.read().clone(), self.is_frozen())
    }
}

pub struct FreezeReadGuard<'a, T> {
    _lock_guard: RwLockReadGuard<'a, ()>,
    data: NonNull<T>,
}

impl<'a, T: 'a> Deref for FreezeReadGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.as_ptr() }
    }
}

pub struct FreezeWriteGuard<'a, T> {
    _lock_guard: RwLockWriteGuard<'a, ()>,
    frozen: &'a AtomicBool,
    data: NonNull<T>,
    marker: PhantomData<&'a mut T>
}

impl<'a, T: 'a> Deref for FreezeWriteGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.as_ptr() }
    }
}

impl<'a, T: 'a> DerefMut for FreezeWriteGuard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.data.as_ptr() }
    }
}

impl<'a, T: 'a> FreezeWriteGuard<'a, T> {
    pub fn freeze(self) -> &'a T {
        self.frozen.store(true, Ordering::Release);

        unsafe { &*self.data.as_ptr() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn freeze_read_test() {
        let lock = FreezeLock::new(250);
        assert_eq!(*lock.read(), 250);
    }

    #[test]
    fn freeze_write_test() {
        let lock = FreezeLock::new(250);

        let mut write_guard = lock.write().unwrap();
        assert_eq!(*write_guard, 250);
        *write_guard += 1;
        drop(write_guard);

        let read_guard = lock.read();
        assert_eq!(*read_guard, 251);
    }

    #[test]
    fn freeze_freeze_test() {
        let lock = FreezeLock::new(250);

        let write_guard = lock.write().unwrap();
        assert_eq!(*write_guard.freeze(), 250);

        assert!(lock.write().is_none())
    }
}
