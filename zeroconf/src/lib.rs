//! `zeroconf` is a cross-platform library that wraps underlying [ZeroConf/mDNS] implementations
//! such as [Bonjour] or [Avahi], providing an easy and idiomatic way to both register and
//! browse services.
//!
//! This crate provides the cross-platform [`MdnsService`] and [`MdnsBrowser`] available for each
//! supported platform as well as platform-specific modules for lower-level access to the mDNS
//! implementation should that be necessary.
//!
//! Most users of this crate need only [`MdnsService`] and [`MdnsBrowser`].
//!
//! # Examples
//!
//! ## Register a service (async/await)
//!
//! `MdnsService::register_async()` provides a `Future` that can be awaited on to register a
//! service via an executor of your choice such as [`tokio`] or [`async-std`].
//!
//! ```no_run
//! use std::thread;
//! use std::time::Duration;
//! use zeroconf::prelude::*;
//! use zeroconf::{MdnsService, ServiceType, TxtRecord};
//!
//! #[tokio::main]
//! pub async fn main() -> zeroconf::Result<()> {
//!     let mut service = MdnsService::new(ServiceType::new("http", "tcp")?, 8080);
//!     let mut txt_record = TxtRecord::new();
//!
//!     txt_record.insert("hello", "world")?;
//!     service.set_txt_record(txt_record);
//!
//!     let result = service.register_async().await;
//!     println!("Service: {:?}", result);
//!
//!     loop {
//!         // do stuff
//!         thread::sleep(Duration::from_nanos(1));
//!     }
//! }
//! ```
//!
//! ## Register a service (callback)
//!
//! When registering a service, you may optionally pass a "context" to pass state through the
//! callback. The only requirement is that this context implements the [`Any`] trait, which most
//! types will automatically. See [`MdnsService`] for more information about contexts.
//!
//! ```no_run
//! use std::any::Any;
//! use std::sync::{Arc, Mutex};
//! use std::time::Duration;
//! use zeroconf::prelude::*;
//! use zeroconf::{MdnsService, ServiceRegistration, ServiceType, TxtRecord};
//!
//! #[derive(Default, Debug)]
//! pub struct Context {
//!     service_name: String,
//! }
//!
//! fn main() {
//!     let mut service = MdnsService::new(ServiceType::new("http", "tcp").unwrap(), 8080);
//!     let mut txt_record = TxtRecord::new();
//!     let context: Arc<Mutex<Context>> = Arc::default();
//!
//!     txt_record.insert("foo", "bar").unwrap();
//!
//!     service.set_registered_callback(Box::new(on_service_registered));
//!     service.set_context(Box::new(context));
//!     service.set_txt_record(txt_record);
//!
//!     let event_loop = service.register().unwrap();
//!
//!     loop {
//!         // calling `poll()` will keep this service alive
//!         event_loop.poll(Duration::from_secs(0)).unwrap();
//!     }
//! }
//!
//! fn on_service_registered(
//!     result: zeroconf::Result<ServiceRegistration>,
//!     context: Option<Arc<dyn Any>>,
//! ) {
//!     let service = result.unwrap();
//!
//!     println!("Service registered: {:?}", service);
//!
//!     let context = context
//!         .as_ref()
//!         .unwrap()
//!         .downcast_ref::<Arc<Mutex<Context>>>()
//!         .unwrap()
//!         .clone();
//!
//!     context.lock().unwrap().service_name = service.name().clone();
//!
//!     println!("Context: {:?}", context);
//!
//!     // do stuff
//! }
//! ```
//!
//! ## Browsing services (async/await)
//!
//! `MdnsBrowser::browse_async()` provides a `Future` that can be awaited on to discover services
//! via an executor of your choice such as [`tokio`] or [`async-std`].
//!
//! ```no_run
//! use zeroconf::prelude::*;
//! use zeroconf::{MdnsBrowser, ServiceType};
//!
//! #[tokio::main]
//! pub async fn main() -> zeroconf::Result<()> {
//!     let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp")?);
//!     loop {
//!         let result = browser.browse_async().await;
//!         println!("Service discovered: {:?}", result.unwrap());
//!     }
//! }
//! ```
//!
//! ## Browsing services (callback)
//!
//! ```no_run
//! use std::any::Any;
//! use std::sync::Arc;
//! use std::time::Duration;
//! use zeroconf::prelude::*;
//! use zeroconf::{MdnsBrowser, ServiceDiscovery, ServiceType};
//!
//! fn main() {
//!     let mut browser = MdnsBrowser::new(ServiceType::new("http", "tcp").unwrap());
//!
//!     browser.set_service_discovered_callback(Box::new(on_service_discovered));
//!
//!     let event_loop = browser.browse().unwrap();
//!
//!     loop {
//!         // calling `poll()` will cause the browser to continue discovering services
//!         event_loop.poll(Duration::from_secs(0)).unwrap();
//!     }
//! }
//!
//! fn on_service_discovered(
//!     result: zeroconf::Result<ServiceDiscovery>,
//!     _context: Option<Arc<dyn Any>>,
//! ) {
//!     println!("Service discovered: {:?}", result.unwrap());
//!
//!     // do stuff
//! }
//! ```
//!
//! [ZeroConf/mDNS]: https://en.wikipedia.org/wiki/Zero-configuration_networking
//! [Bonjour]: https://en.wikipedia.org/wiki/Bonjour_(software)
//! [Avahi]: https://en.wikipedia.org/wiki/Avahi_(software)
//! [`MdnsService`]: type.MdnsService.html
//! [`MdnsBrowser`]: type.MdnsBrowser.html
//! [`Any`]: https://doc.rust-lang.org/std/any/trait.Any.html
//! [`async-std`]: https://async.rs/
//! [`tokio`]: https://tokio.rs/

#![allow(clippy::needless_doctest_main)]
#[macro_use]
extern crate serde;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate zeroconf_macros;
#[cfg(target_os = "linux")]
extern crate avahi_sys;
#[cfg(target_vendor = "apple")]
extern crate bonjour_sys;
#[macro_use]
extern crate derive_getters;
#[macro_use]
extern crate log;
#[macro_use]
extern crate derive_new;

#[macro_use]
#[cfg(test)]
#[allow(unused_imports)]
extern crate maplit;

#[macro_use]
mod macros;
mod ffi;
mod interface;
mod service_type;
#[cfg(test)]
mod tests;

pub mod browser;
pub mod error;
pub mod event_loop;
pub mod prelude;
pub mod service;
pub mod txt_record;
pub mod util;

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_vendor = "apple")]
pub mod macos;

pub use browser::{ServiceDiscoveredCallback, ServiceDiscovery};
pub use interface::*;
pub use service::{ServiceRegisteredCallback, ServiceRegistration};
pub use service_type::*;

/// Type alias for the platform-specific mDNS browser implementation
#[cfg(target_os = "linux")]
pub type MdnsBrowser = linux::browser::AvahiMdnsBrowser;
/// Type alias for the platform-specific mDNS browser implementation
#[cfg(target_vendor = "apple")]
pub type MdnsBrowser = macos::browser::BonjourMdnsBrowser;

/// Type alias for the platform-specific mDNS service implementation
#[cfg(target_os = "linux")]
pub type MdnsService = linux::service::AvahiMdnsService;
/// Type alias for the platform-specific mDNS service implementation
#[cfg(target_vendor = "apple")]
pub type MdnsService = macos::service::BonjourMdnsService;

/// Type alias for the platform-specific structure responsible for polling the mDNS event loop
#[cfg(target_os = "linux")]
pub type EventLoop = linux::event_loop::AvahiEventLoop;
/// Type alias for the platform-specific structure responsible for polling the mDNS event loop
#[cfg(target_vendor = "apple")]
pub type EventLoop = macos::event_loop::BonjourEventLoop;

/// Type alias for the platform-specific structure responsible for storing and accessing TXT
/// record data
#[cfg(target_os = "linux")]
pub type TxtRecord = linux::txt_record::AvahiTxtRecord;
/// Type alias for the platform-specific structure responsible for storing and accessing TXT
/// record data
#[cfg(target_vendor = "apple")]
pub type TxtRecord = macos::txt_record::BonjourTxtRecord;

/// Result type for this library
pub type Result<T> = std::result::Result<T, error::Error>;
