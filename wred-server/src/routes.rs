use actix_web::{delete, get, post, web, HttpResponse, Responder};

#[get("/all")]
async fn get_logs(data: web::Data<super::state::AppState>) -> impl Responder {
    let data = data.into_inner();
    let logs = data.logs.lock().await;
    println!("{logs:?}");
    let resp: Vec<_> = logs
        .iter()
        .map(|(&id, v)| wred_server::LogEntryPartial {
            id,
            addr: v.addr,
            last_updated: v.last_updated,
            is_saved: data.config.log_dir.join(format!("{}.log", id)).exists(),
        })
        .collect();
    postcard::to_allocvec(&resp).map_or_else(
        |e| HttpResponse::InternalServerError().body(format!("Failed to serialise: {}", e)),
        |v| HttpResponse::Ok().body(v),
    )
}

#[get("/{id:[[:digit:]]+}")]
async fn get_log(
    path: web::Path<String>,
    data: web::Data<super::state::AppState>,
) -> impl Responder {
    let id: u64 = path.into_inner().parse().unwrap();
    let data = data.into_inner();
    let logs = data.logs.lock().await;
    logs.get(&id).map_or_else(
        || HttpResponse::NotFound().finish(),
        |v| HttpResponse::Ok().body(postcard::to_allocvec(&v).unwrap()),
    )
}

#[delete("/{id:[[:digit:]]+}")]
async fn delete_log(
    path: web::Path<String>,
    data: web::Data<super::state::AppState>,
    body: web::Bytes,
) -> impl Responder {
    let id: u64 = path.into_inner().parse().unwrap();
    let data = data.into_inner();
    let mut logs = data.logs.lock().await;
    postcard::from_bytes(&body).map_or_else(
        move |v| HttpResponse::BadRequest().body(format!("Failed to deserialise: {}", v)),
        |v: String| {
            if v == data.config.secret {
                logs.remove(&id).map_or_else(
                    || HttpResponse::NotFound().finish(),
                    |v| {
                        let _e =
                            std::fs::remove_file(data.config.log_dir.join(format!("{}.log", id)));
                        postcard::to_allocvec(&v).map_or_else(
                            |e| {
                                HttpResponse::InternalServerError()
                                    .body(format!("Failed to serialise: {}", e))
                            },
                            |v| HttpResponse::Ok().body(v),
                        )
                    },
                )
            } else {
                HttpResponse::Unauthorized().finish()
            }
        },
    )
}

#[post("/{id:[[:digit:]]+}")]
async fn save_log(
    path: web::Path<String>,
    data: web::Data<super::state::AppState>,
    body: web::Bytes,
) -> impl Responder {
    let id: u64 = path.into_inner().parse().unwrap();
    let data = data.into_inner();
    let logs = data.logs.lock().await;
    postcard::from_bytes(&body).map_or_else(
        move |v| HttpResponse::BadRequest().body(format!("Failed to deserialise: {}", v)),
        |v: String| {
            if v == data.config.secret {
                logs.get(&id).map_or_else(
                    || HttpResponse::NotFound().finish(),
                    |v| {
                        postcard::to_allocvec(&(id, v)).map_or_else(
                            |e| {
                                HttpResponse::InternalServerError()
                                    .body(format!("Failed to serialise: {}", e))
                            },
                            |v| {
                                std::fs::write(data.config.log_dir.join(format!("{id}.log")), &v)
                                    .map_or_else(
                                        |e| {
                                            HttpResponse::InternalServerError()
                                                .body(format!("Failed to save log: {}", e))
                                        },
                                        |_| HttpResponse::Ok().finish(),
                                    )
                            },
                        )
                    },
                )
            } else {
                HttpResponse::Unauthorized().finish()
            }
        },
    )
}
