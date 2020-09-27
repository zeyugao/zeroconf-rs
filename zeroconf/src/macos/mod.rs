//! macOS-specific ZeroConf bindings
//!
//! This module wraps the [Bonjour] mDNS implementation which is distributed with macOS.
//!
//! [Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)

pub(crate) mod browser;
pub(crate) mod constants;
pub(crate) mod service;

pub mod bonjour_util;
pub mod event_loop;
pub mod service_ref;
pub mod txt_record;
