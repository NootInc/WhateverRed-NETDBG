#![warn(
    warnings,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    unused_extern_crates,
    rust_2021_compatibility
)]
#![allow(clippy::module_name_repetitions)]

use sequence_generator::sequence_generator;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct LogEntryPartial {
    pub id: u64,
    pub last_updated: u64,
    pub addr: std::net::SocketAddr,
    pub is_saved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub last_updated: u64,
    pub addr: std::net::SocketAddr,
    pub data: String,
}

#[must_use]
pub fn get_id_props() -> sequence_generator::SequenceProperties {
    sequence_generator::SequenceProperties::new(std::time::UNIX_EPOCH, 10, 500, 12, 3, 1, 1500)
}
