pub use spin::lock_api::{Mutex, MutexGuard};
pub type MappedMutexGuard<'a, T, U> = lock_api::MappedMutexGuard<'a, spin::Mutex<T>, U>;

pub use spin::lock_api::{RwLock, RwLockReadGuard, RwLockWriteGuard};
pub type MappedRwLockReadGuard<'a, T, U> = lock_api::MappedRwLockReadGuard<'a, spin::RwLock<T>, U>;
pub type MappedRwLockWriteGuard<'a, T, U> =
    lock_api::MappedRwLockWriteGuard<'a, spin::RwLock<T>, U>;

pub type Lock = Mutex<()>;
pub type LockGuard = MutexGuard<'static, ()>;
pub type MappedLockGuard<T> = MappedMutexGuard<'static, (), T>;

pub fn lock_nb<T>(mutex: &spin::Mutex<T>) -> spin::MutexGuard<T> {
    match mutex.try_lock() {
        Some(guard) => guard,
        None => panic!("Tried to lock a locked mutex!"),
    }
}
