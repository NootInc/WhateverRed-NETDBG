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
) -> std::io::Result<HttpResponse> {
    let id: u64 = path.into_inner().parse().unwrap();
    let data = data.into_inner();
    let mut logs = data.logs.lock().await;
    let secret: String = postcard::from_bytes(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    if secret == data.config.secret {
        if logs.remove(&id).is_none() {
            Ok(HttpResponse::NotFound().finish())
        } else {
            let path = data.config.log_dir.join(format!("{}.log", id));
            if path.exists() {
                tokio::fs::remove_file(path).await?;
            }
            Ok(HttpResponse::Ok().finish())
        }
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}

#[post("/{id:[[:digit:]]+}")]
async fn save_log(
    path: web::Path<String>,
    data: web::Data<super::state::AppState>,
    body: web::Bytes,
) -> std::io::Result<HttpResponse> {
    let id: u64 = path.into_inner().parse().unwrap();
    let data = data.into_inner();
    let logs = data.logs.lock().await;
    let secret: String = postcard::from_bytes(&body)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    if secret == data.config.secret {
        if let Some(v) = logs.get(&id) {
            tokio::fs::write(
                data.config.log_dir.join(format!("{id}.log")),
                &postcard::to_allocvec(&(id, v))
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?,
            )
            .await?;
            Ok(HttpResponse::Ok().finish())
        } else {
            Ok(HttpResponse::NotFound().finish())
        }
    } else {
        Ok(HttpResponse::Unauthorized().finish())
    }
}
