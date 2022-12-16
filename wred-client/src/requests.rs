pub fn delete_log(base_url: &str, id: u64, secret: &str, ctx: egui::Context) {
    ehttp::fetch(
        ehttp::Request {
            method: "DELETE".to_owned(),
            ..ehttp::Request::post(
                format!("{base_url}/{id}"),
                postcard::to_allocvec(secret).unwrap(),
            )
        },
        move |response| {
            if let Err(e) = response {
                eprintln!("Error: {e}");
            }
            ctx.request_repaint();
        },
    );
}

pub fn save_log(base_url: &str, id: u64, secret: &str, ctx: egui::Context) {
    let request = ehttp::Request::post(
        format!("{base_url}/{id}"),
        postcard::to_allocvec(secret).unwrap(),
    );
    ehttp::fetch(request, move |response| {
        if let Err(e) = response {
            eprintln!("Error: {e}");
        }
        ctx.request_repaint();
    });
}

pub fn get_logs(
    base_url: &str,
    sender: poll_promise::Sender<Result<Vec<wred_server::LogEntryPartial>, String>>,
    ctx: egui::Context,
) {
    ehttp::fetch(
        ehttp::Request::get(format!("{base_url}/all")),
        move |response| {
            let ent =
                response.and_then(|v| postcard::from_bytes(&v.bytes).map_err(|e| e.to_string()));
            sender.send(ent);
            ctx.request_repaint();
        },
    );
}

pub fn get_log(
    base_url: &str,
    id: u64,
    sender: poll_promise::Sender<Result<String, String>>,
    ctx: egui::Context,
) {
    ehttp::fetch(
        ehttp::Request::get(format!("{base_url}/{id}")),
        move |response| {
            let ent = response.map(|v| String::from_utf8_lossy(&v.bytes).into_owned());
            sender.send(ent);
            ctx.request_repaint();
        },
    );
}
