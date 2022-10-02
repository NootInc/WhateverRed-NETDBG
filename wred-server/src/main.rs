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

mod log_service;
mod routes;
mod state;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let f = std::fs::File::open("./config.ron").expect("Failed opening config");
    let config: state::ServerConfig = ron::de::from_reader(f).unwrap();
    let state = web::Data::new(state::AppState {
        config,
        ..Default::default()
    });
    let _e = std::fs::create_dir_all(&state.config.log_dir);
    for ent in std::fs::read_dir(&state.config.log_dir).unwrap() {
        let path = ent.unwrap().path();
        if path.is_file() {
            let data = std::fs::read(path).unwrap();
            let ent: (u64, wred_server::LogEntry) = postcard::from_bytes(&data).unwrap();
            state.logs.lock().await.insert(ent.0, ent.1);
        }
    }

    log_service::start_log_receiver(state.clone());
    let bind = (state.config.ip.clone(), state.config.api_port);
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
