//! Rust port of [imbushuo/BluetoothAudioCodecInspector][upstream]. Inspects
//! the active Bluetooth A2DP audio codec on Windows by subscribing to the
//! `Microsoft.Windows.Bluetooth.BthA2dp` ETW provider, since Windows does
//! not expose the active codec through any public API.
//!
//! The [`A2dpCodec`] type and its codec ID -> name mapping are available on
//! all platforms — useful if you have raw codec IDs from another source.
//! The [`watch`] function and [`Watcher`] guard are Windows only because
//! they depend on ETW.
//!
//! # Example
//!
//! ```no_run
//! # #[cfg(windows)] {
//! use btcodec_inspector::watch;
//!
//! // Requires the process to be running elevated.
//! let _watcher = watch(|codec| {
//!     println!("Codec: {}", codec);
//! }).expect("failed to start ETW session");
//!
//! // Keep `_watcher` alive while you want events. Drop to stop.
//! std::thread::sleep(std::time::Duration::from_secs(60));
//! # }
//! ```
//!
//! The callback is `FnMut`, so captured state can be updated directly:
//!
//! ```no_run
//! # #[cfg(windows)] {
//! use btcodec_inspector::{watch, A2dpCodec};
//!
//! let mut history: Vec<A2dpCodec> = Vec::new();
//! let _watcher = watch(move |codec| {
//!     history.push(codec); // no Mutex / Arc needed
//! }).unwrap();
//! # }
//! ```
//!
//! [upstream]: https://github.com/imbushuo/BluetoothAudioCodecInspector

mod codec;
pub use codec::A2dpCodec;

#[cfg(windows)]
mod watcher;
#[cfg(windows)]
pub use watcher::{watch, Error, Watcher};
