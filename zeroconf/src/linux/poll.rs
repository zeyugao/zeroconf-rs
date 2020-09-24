//! Rust friendly `AvahiSimplePoll` wrappers/helpers

use super::avahi_util;
use avahi_sys::{
    avahi_simple_poll_free, avahi_simple_poll_loop, avahi_simple_poll_new, AvahiSimplePoll,
};
use std::ptr;

/// Wraps the `AvahiSimplePoll` type from the raw Avahi bindings.
///
/// This struct allocates a new `*mut AvahiSimplePoll` when `ManagedAvahiClient::new()` is invoked
/// and calls the Avahi function responsible for freeing the poll on `trait Drop`.
#[derive(Debug)]
pub struct ManagedAvahiSimplePoll {
    pub(super) poll: *mut AvahiSimplePoll,
}

impl ManagedAvahiSimplePoll {
    /// Initializes the underlying `*mut AvahiSimplePoll` and verifies it was created; returning
    /// `Err(String)` if unsuccessful
    pub fn new() -> Result<Self, String> {
        let poll = unsafe { avahi_simple_poll_new() };
        if poll == ptr::null_mut() {
            Err("could not initialize AvahiSimplePoll".to_string())
        } else {
            Ok(Self { poll })
        }
    }

    /// Delegate function for [`avahi_simple_poll_loop()`].
    ///
    /// [`avahi_simple_poll_loop()`]: https://avahi.org/doxygen/html/simple-watch_8h.html#a14b4cb29832e8c3de609d4c4e5611985
    pub fn start_loop(&self) -> Result<(), String> {
        let err = unsafe { avahi_simple_poll_loop(self.poll) };
        if err != 0 {
            Err(format!(
                "could not start AvahiSimplePoll: {}",
                avahi_util::get_error(err)
            ))
        } else {
            Ok(())
        }
    }
}

impl Drop for ManagedAvahiSimplePoll {
    fn drop(&mut self) {
        unsafe { avahi_simple_poll_free(self.poll) };
    }
}
