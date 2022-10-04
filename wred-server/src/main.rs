#![warn(
    warnings,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo,
    unused_extern_crates,
    rust_2021_compatibility
)]
#![allow(clippy::module_name_repetitions)]

use actix_web::{web, App, HttpServer};
use tokio::io::AsyncReadExt;

mod log_service;
mod routes;
mod state;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut f = tokio::fs::File::open("./config.ron")
        .await
        .expect("Failed opening config");
    let mut s = String::new();
    f.read_to_string(&mut s).await.unwrap();
    let config: state::ServerConfig = ron::de::from_str(&s).unwrap();
    let state = web::Data::new(state::AppState {
        config,
        ..Default::default()
    });
    let _e = tokio::fs::create_dir_all(&state.config.log_dir).await;
    let mut rd = tokio::fs::read_dir(&state.config.log_dir).await.unwrap();
    loop {
        match rd.next_entry().await.unwrap() {
            None => break,
            Some(ent) => {
                let path = ent.path();
                if path.is_file() {
                    let data = tokio::fs::read(path).await.unwrap();
                    let ent: (u64, wred_server::LogEntry) = postcard::from_bytes(&data).unwrap();
                    state.logs.lock().unwrap().insert(ent.0, ent.1);
                }
            }
        }
    }

    let bind = (state.config.ip.clone(), state.config.api_port);
    log_service::start_log_receiver(state.clone());

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(routes::get_logs)
            .service(routes::get_log)
            .service(routes::delete_log)
            .service(routes::save_log)
            .service(actix_files::Files::new("/", "./dist").index_file("index.html"))
    })
    .bind(bind)?
    .run()
    .await
}
