use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use actix_web::web;
use sequence_generator::sequence_generator;
use tokio::io::AsyncReadExt;
use wred_server::LogEntry;

fn generate_id() -> (sequence_generator::SequenceProperties, u64) {
    let properties = wred_server::get_id_props();
    let id = sequence_generator::generate_id(&properties).unwrap();
    (properties, id)
}

async fn handle_connection(
    mut stream: tokio::net::TcpStream,
    addr: std::net::SocketAddr,
    log_entries: Arc<Mutex<HashMap<u64, LogEntry>>>,
) {
    stream.readable().await.unwrap();
    println!("Incoming connection from: {}", addr.ip());

    let mut buf = Vec::new();
    while let Ok(n) = stream.read_buf(&mut buf).await {
        if n == 0 {
            break;
        }

        let (properties, id) = generate_id();
        let v = wred_server::LogEntry {
            last_updated: sequence_generator::decode_id_unix_epoch_micros(id, &properties),
            addr,
            data: String::from_utf8_lossy(&buf).to_string(),
        };
        println!("{v:#?}");

        let mut logs = log_entries.lock().unwrap();
        if let Some((_, ent)) = logs
            .iter_mut()
            .find(|(_, e)| e.addr.ip() == addr.ip() && v.last_updated - e.last_updated < 60_000_000)
        {
            ent.last_updated = v.last_updated;
            ent.data += &v.data;
        } else {
            logs.insert(id, v);
        }

        buf.clear();
    }
}

pub fn start_log_receiver(state: web::Data<crate::state::AppState>) {
    let bind = (state.config.ip.clone(), state.config.logger_port);

    tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(bind).await.unwrap();

        loop {
            let (stream, addr) = listener.accept().await.unwrap();
            let log_entries = state.logs.clone();

            tokio::spawn(async move {
                handle_connection(stream, addr, log_entries).await;
            });
        }
    });
}
