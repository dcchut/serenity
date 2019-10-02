#[macro_use]
pub mod macros;

pub mod prelude;

mod rwlock_ext;

pub use self::rwlock_ext::RwLockExt;

pub use parking_lot::RwLock as SyncRwLock;
pub use async_std::sync::RwLock as AsyncRwLock;

#[cfg(feature = "gateway")]
pub mod ws_impl;

#[cfg(feature = "voice")]
mod timer;

#[cfg(feature = "voice")]
pub use self::timer::Timer;
