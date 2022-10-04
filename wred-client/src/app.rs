use std::collections::HashMap;

use egui::{
    collapsing_header::CollapsingState, Button, CentralPanel, Color32, Frame, Label, Layout,
    RichText, ScrollArea, Sense, TextEdit,
};
use poll_promise::Promise;
use sequence_generator::sequence_generator;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct WRedNetDbgApp {
    base_url: String,
    secret: String,
    #[serde(skip)]
    log_cache: HashMap<u64, Promise<Result<String, String>>>,
    #[serde(skip)]
    log_cache_ents: Option<Promise<Result<Vec<wred_server::LogEntryPartial>, String>>>,
    #[serde(skip)]
    formatter: timeago::Formatter,
}

impl Default for WRedNetDbgApp {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let base_url = web_sys::window()
            .and_then(|v| v.document())
            .and_then(|v| v.location())
            .and_then(|v| Some(v.origin()))
            .unwrap()
            .unwrap();
        #[cfg(not(target_arch = "wasm32"))]
        let base_url = "http://localhost:8080".to_string();
        Self {
            base_url,
            secret: String::new(),
            log_cache: HashMap::default(),
            log_cache_ents: None,
            formatter: timeago::Formatter::with_language(timeago::English),
        }
    }
}

impl WRedNetDbgApp {
    #[must_use]
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        cc.egui_ctx.set_fonts(crate::style::get_fonts());

        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self::default()
    }
}

impl eframe::App for WRedNetDbgApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.set_min_height(25.0);

            ui.centered_and_justified(|ui| {
                egui::menu::bar(ui, |ui| {
                    #[cfg(target_os = "macos")]
                    ui.add_space(60.0);
                    ui.add(
                        TextEdit::singleline(&mut self.secret)
                            .id_source("admin_secret")
                            .password(true)
                            .hint_text("Admin Secret"),
                    );
                    ui.add(
                        TextEdit::singleline(&mut self.base_url)
                            .id_source("base_url")
                            .hint_text("Base URL"),
                    );

                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("\u{1F504}").clicked() {
                            self.log_cache.clear();
                            self.log_cache_ents = None;
                        }
                        ui.separator();
                        ui.label(RichText::new("NETDBG").small().monospace());
                        ui.label("WhateverRed");
                    });
                });
            });
        });

        let cached_promise = self.log_cache_ents.get_or_insert_with(|| {
            let ctx = ctx.clone();
            let (sender, promise) = Promise::new();
            let request = ehttp::Request::get(format!("{}/all", self.base_url));
            ehttp::fetch(request, move |response| {
                let ent = response
                    .and_then(|v| postcard::from_bytes(&v.bytes).map_err(|e| e.to_string()));
                sender.send(ent);
                ctx.request_repaint();
            });
            promise
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| match cached_promise.ready() {
                None => {
                    ui.spinner();
                }
                Some(Err(e)) => {
                    ui.colored_label(Color32::RED, RichText::new(e));
                }
                Some(Ok(ents)) => {
                    ui.set_width(ui.available_width());

                    for ent in ents {
                        let cached_promise = self.log_cache.entry(ent.id).or_insert_with(|| {
                            let ctx = ctx.clone();
                            let (sender, promise) = Promise::new();
                            let request =
                                ehttp::Request::get(format!("{}/{}", self.base_url, ent.id));
                            ehttp::fetch(request, move |response| {
                                let ent = response.and_then(|v| {
                                    postcard::from_bytes(&v.bytes).map_err(|e| e.to_string())
                                });
                                sender.send(ent);
                                ctx.request_repaint();
                            });
                            promise
                        });
                        Frame::group(&ctx.style()).show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            CollapsingState::load_with_default_open(
                                ctx,
                                ui.make_persistent_id(ent.id),
                                false,
                            )
                            .show_header(ui, |ui| {
                                egui::menu::bar(ui, |ui| {
                                    ui.add(Label::new(ent.addr.to_string()).sense(Sense::click()))
                                        .context_menu(|ui| {
                                            if ui.button("\u{1F5D0} Copy IP").clicked() {
                                                ui.output().copied_text = ent.addr.to_string();
                                                ui.close_menu();
                                            }
                                        });
                                    let props = wred_server::get_id_props();

                                    #[cfg(not(target_arch = "wasm32"))]
                                    #[allow(clippy::cast_possible_truncation)]
                                    let cur = std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap()
                                        .as_micros()
                                        as u64;
                                    #[cfg(target_arch = "wasm32")]
                                    #[allow(clippy::cast_possible_truncation)]
                                    let cur = (js_sys::Date::new_0().get_time() * 1000.0) as u64;
                                    let micros = sequence_generator::decode_id_unix_epoch_micros(
                                        ent.id, &props,
                                    );
                                    let d = cur - micros;
                                    let text =
                                        self.formatter.convert(std::time::Duration::from_micros(d));
                                    ui.label(RichText::new(text).weak());

                                    let d = cur - ent.last_updated;
                                    ui.separator();
                                    ui.label(RichText::new("updated").weak());
                                    let text =
                                        self.formatter.convert(std::time::Duration::from_micros(d));
                                    ui.label(RichText::new(text).weak());

                                    ui.with_layout(
                                        Layout::right_to_left(egui::Align::Center),
                                        |ui| match cached_promise.ready() {
                                            None => {
                                                ui.spinner();
                                            }
                                            Some(Err(_)) => {
                                                let _e = ui.button("\u{1F5D9}");
                                            }
                                            Some(Ok(ent_full)) => {
                                                if ui.button("\u{2B8B} Save").clicked() {
                                                    ui.output().copied_text = ent_full.clone();
                                                }
                                                if ui.button("\u{1F5D0} Copy").clicked() {
                                                    ui.output().copied_text = ent_full.clone();
                                                }
                                                let resp = ui.add_enabled(
                                                    !self.secret.is_empty(),
                                                    Button::new("\u{274C} Discard"),
                                                );
                                                let id = resp.id.with("discard_confirmation");
                                                egui::popup::popup_below_widget(
                                                    ui,
                                                    id,
                                                    &resp,
                                                    |ui| {
                                                        ui.set_min_width(80.0);
                                                        ui.label("Are you sure?");
                                                        ui.horizontal(|ui| {
                                                            if ui.button("Yes").clicked() {
                                                                ui.memory().close_popup();
                                                                let ctx = ctx.clone();
                                                                ehttp::fetch(
                                                                    ehttp::Request {
                                                                        method: "DELETE".to_owned(),
                                                                        url: format!(
                                                                            "{}/{}",
                                                                            self.base_url, ent.id
                                                                        ),
                                                                        body:
                                                                            postcard::to_allocvec(
                                                                                &self.secret,
                                                                            )
                                                                            .unwrap(),
                                                                        ..ehttp::Request::get("")
                                                                    },
                                                                    move |response| {
                                                                        if let Err(e) = response {
                                                                            eprintln!(
                                                                                "Error: {}",
                                                                                e
                                                                            );
                                                                        }
                                                                        ctx.request_repaint();
                                                                    },
                                                                );
                                                            }
                                                            if ui.button("No").clicked() {
                                                                ui.memory().close_popup();
                                                            }
                                                        });
                                                    },
                                                );
                                                if resp.clicked() {
                                                    ui.memory().open_popup(id);
                                                }
                                                let resp = ui.add_enabled(
                                                    !self.secret.is_empty(),
                                                    Button::new("\u{2705} Keep"),
                                                );
                                                let id = resp.id.with("keep_confirmation");

                                                let save = || {
                                                    let ctx = ctx.clone();
                                                    let request = ehttp::Request::post(
                                                        format!("{}/{}", self.base_url, ent.id),
                                                        postcard::to_allocvec(&self.secret)
                                                            .unwrap(),
                                                    );
                                                    ehttp::fetch(request, move |response| {
                                                        if let Err(e) = response {
                                                            eprintln!("Error: {}", e);
                                                        }
                                                        ctx.request_repaint();
                                                    });
                                                };
                                                egui::popup::popup_below_widget(
                                                    ui,
                                                    id,
                                                    &resp,
                                                    |ui| {
                                                        ui.set_min_width(80.0);
                                                        ui.label("Are you sure?");
                                                        ui.horizontal(|ui| {
                                                            if ui.button("Yes").clicked() {
                                                                ui.memory().close_popup();
                                                                save();
                                                            }
                                                            if ui.button("No").clicked() {
                                                                ui.memory().close_popup();
                                                            }
                                                        });
                                                    },
                                                );
                                                if resp.clicked() {
                                                    if ent.is_saved {
                                                        ui.memory().open_popup(id);
                                                    } else {
                                                        save();
                                                    }
                                                }
                                            }
                                        },
                                    );
                                });
                            })
                            .body(|ui| {
                                ui.set_width(ui.available_width());

                                match cached_promise.ready() {
                                    None => {
                                        ui.spinner();
                                    }
                                    Some(Err(e)) => {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new("\u{1F5D9}"));
                                            ui.label(e);
                                        });
                                    }
                                    Some(Ok(ent)) => {
                                        ui.add(
                                            TextEdit::multiline(&mut ent.trim())
                                                .code_editor()
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(1)
                                                .hint_text("Nothing to see here"),
                                        );
                                    }
                                }
                            })
                            .0
                            .context_menu(|ui| {
                                if ui.button("\u{1F5D0} Copy ID").clicked() {
                                    ui.output().copied_text = ent.id.to_string();
                                    ui.close_menu();
                                }
                            });
                        });
                    }
                }
            });
        });
    }
}
