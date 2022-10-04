use actix_web::web;
use sequence_generator::sequence_generator;

fn generate_id() -> (sequence_generator::SequenceProperties, u64) {
    let properties = wred_server::get_id_props();
    let id = sequence_generator::generate_id(&properties).unwrap();
    (properties, id)
}

pub async fn start_log_receiver(state: web::Data<crate::state::AppState>) {
    let bind = (state.config.ip.clone(), state.config.logger_port);
    let listener = tokio::net::TcpListener::bind(bind).await.unwrap();

    tokio::spawn(async move {
        loop {
            let (stream, addr) = listener.accept().await.unwrap();
            let log_entries = state.logs.clone();

            tokio::spawn(async move {
                tokio::time::timeout(std::time::Duration::from_secs(30), stream.readable())
                    .await
                    .unwrap()
                    .unwrap();
                let mut buf = Vec::new();
                let e: std::io::Result<()> = loop {
                    match stream.try_read_buf(&mut buf) {
                        Ok(0) => break Ok(()),
                        Ok(_) => {
                            let (properties, id) = generate_id();
                            let v = wred_server::LogEntry {
                                last_updated: sequence_generator::decode_id_unix_epoch_micros(
                                    id,
                                    &properties,
                                ),
                                addr,
                                data: String::from_utf8_lossy(&buf).to_string(),
                            };
                            buf.clear();

                            let mut logs = log_entries.lock().unwrap();
                            if let Some(ent) = logs.values_mut().find(|e| {
                                e.addr.ip() == addr.ip()
                                    && v.last_updated - e.last_updated < 60_000_000
                            }) {
                                ent.last_updated = v.last_updated;
                                ent.data += &v.data;
                            } else {
                                logs.insert(id, v);
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            continue;
                        }
                        Err(e) => {
                            break Err(e);
                        }
                    };
                };
                e.unwrap();
            });
        }
    });
}
