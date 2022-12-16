#[cfg(not(target_arch = "wasm32"))]
pub fn cur_micros() -> u64 {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            (js_sys::Date::new_0().get_time() * 1000.0) as u64
        } else {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_micros() as u64
        }
    }
}

pub fn base_url() -> String {
    cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            web_sys::window()
                    .and_then(|v| v.document())
                    .and_then(|v| v.location())
                    .and_then(|v| Some(v.origin()))
                    .unwrap()
                    .unwrap()
        } else {
            "http://localhost:8080".to_owned()
        }
    }
}
