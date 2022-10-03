use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ServerConfig {
    pub ip: String,
    pub api_port: u16,
    pub logger_port: u16,
    pub secret: String,
    pub log_dir: PathBuf,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub config: ServerConfig,
    pub logs: Arc<Mutex<HashMap<u64, wred_server::LogEntry>>>,
}
