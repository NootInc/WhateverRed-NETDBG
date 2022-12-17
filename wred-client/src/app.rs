use std::collections::HashMap;

use egui::{
    collapsing_header::CollapsingState, Align, Button, CentralPanel, Color32, ComboBox, Frame,
    Label, Layout, RichText, ScrollArea, Sense, TextEdit, TopBottomPanel,
};
use poll_promise::Promise;
use sequence_generator::sequence_generator;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Copy, Clone, PartialEq, Eq)]
pub enum SortBy {
    CreationDate,
    IPAddress,
    LastUpdated,
}

impl ToString for SortBy {
    fn to_string(&self) -> String {
        match self {
            Self::IPAddress => "IP Address",
            Self::CreationDate => "Creation Date",
            Self::LastUpdated => "Last Updated",
        }
        .to_owned()
    }
}

#[derive(Deserialize, Serialize)]
pub struct WRedNetDbgApp {
    base_url: String,
    secret: String,
    show_ips: bool,
    show_base: bool,
    sort_by: SortBy,
    sort_ascending: bool,
    #[serde(skip)]
    log_cache: HashMap<u64, Promise<Result<String, String>>>,
    #[serde(skip)]
    log_cache_ents: Option<Promise<Result<Vec<wred_server::LogEntryPartial>, String>>>,
    #[serde(skip)]
    formatter: timeago::Formatter,
}

impl Default for WRedNetDbgApp {
    fn default() -> Self {
        Self {
            base_url: crate::utils::base_url(),
            secret: String::new(),
            show_ips: true,
            show_base: false,
            sort_by: SortBy::CreationDate,
            sort_ascending: false,
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
        cc.storage.map_or_else(Self::default, |storage| {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        })
    }
}

impl eframe::App for WRedNetDbgApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.set_height(25.0);

            ui.centered_and_justified(|ui| {
                egui::menu::bar(ui, |ui| {
                    #[cfg(target_os = "macos")]
                    ui.add_space(60.0);

                    ui.add(
                        TextEdit::singleline(&mut self.secret)
                            .desired_width(150.0)
                            .password(true)
                            .hint_text("Admin Secret"),
                    );

                    ui.add(
                        TextEdit::singleline(&mut self.base_url)
                            .desired_width(150.0)
                            .password(!self.show_base)
                            .hint_text("Base URL"),
                    );
                    ui.toggle_value(&mut self.show_base, "\u{1F441}");

                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if ui.button("\u{1F504}").clicked() {
                            self.log_cache.clear();
                            self.log_cache_ents = None;
                        }

                        if ui.button("Discard unsaved").clicked() {
                            if let Some(Ok(ents)) = &self.log_cache_ents.and_then(|v| v.ready()) {
                                for ent in ents.iter().filter(|v| !v.is_saved) {
                                    self.log_cache.remove(&ent.id);
                                    crate::requests::delete_log(
                                        &self.base_url,
                                        ent.id,
                                        &self.secret,
                                        ctx.clone(),
                                    );
                                }
                            }
                            self.log_cache_ents = None;
                        }

                        ui.separator();
                        ui.checkbox(&mut self.show_ips, "IPs shown");
                        ui.separator();

                        ui.toggle_value(&mut self.sort_ascending, "\u{2B06}");
                        ComboBox::from_id_source("sort_by")
                            .selected_text(self.sort_by.to_string())
                            .show_ui(ui, |ui| {
                                for v in
                                    [SortBy::CreationDate, SortBy::IPAddress, SortBy::LastUpdated]
                                {
                                    ui.selectable_value(&mut self.sort_by, v, v.to_string());
                                }
                            });
                        ui.label("Sort by");
                    });
                });
            });
        });

        let log_cache_ents = self.log_cache_ents.get_or_insert_with(|| {
            let (sender, promise) = Promise::new();
            crate::requests::get_logs(&self.base_url, sender, ctx.clone());
            promise
        });

        CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| match log_cache_ents.ready() {
                None => {
                    ui.spinner();
                }
                Some(Err(e)) => {
                    ui.colored_label(Color32::RED, RichText::new(e));
                }
                Some(Ok(ents)) => {
                    let mut ents = ents.iter().collect::<Vec<_>>();
                    ents.sort_by(|a, b| match self.sort_by {
                        SortBy::CreationDate => {
                            if self.sort_ascending {
                                a.id.cmp(&b.id)
                            } else {
                                b.id.cmp(&a.id)
                            }
                        }
                        SortBy::IPAddress => {
                            if self.sort_ascending {
                                a.addr.ip().cmp(&b.addr.ip())
                            } else {
                                b.addr.ip().cmp(&a.addr.ip())
                            }
                        }
                        SortBy::LastUpdated => {
                            if self.sort_ascending {
                                a.last_updated.cmp(&b.last_updated)
                            } else {
                                b.last_updated.cmp(&a.last_updated)
                            }
                        }
                    });
                    for ent in ents {
                        let cached_promise = self.log_cache.entry(ent.id).or_insert_with(|| {
                            let (sender, promise) = Promise::new();
                            crate::requests::get_log(&self.base_url, ent.id, sender, ctx.clone());
                            promise
                        });

                        Frame::group(ui.style()).show(ui, |ui| {
                            CollapsingState::load_with_default_open(
                                ctx,
                                ui.make_persistent_id(ent.id),
                                false,
                            )
                            .show_header(ui, |ui| {
                                egui::menu::bar(ui, |ui| {
                                    ui.add(
                                        Label::new(if self.show_ips {
                                            ent.addr.to_string()
                                        } else {
                                            "IP Hidden".to_owned()
                                        })
                                        .sense(Sense::click()),
                                    )
                                    .context_menu(|ui| {
                                        if ui.button("\u{1F5D0} Copy IP").clicked() {
                                            ui.output().copied_text = ent.addr.to_string();
                                            ui.close_menu();
                                        }
                                    });

                                    let props = wred_server::get_id_props();
                                    let cur_micros = crate::utils::cur_micros();
                                    let micros = sequence_generator::decode_id_unix_epoch_micros(
                                        ent.id, &props,
                                    );
                                    ui.label(
                                        RichText::new(self.formatter.convert(
                                            std::time::Duration::from_micros(cur_micros - micros),
                                        ))
                                        .weak(),
                                    );

                                    ui.separator();

                                    ui.label(RichText::new("updated").weak());
                                    ui.label(
                                        RichText::new(self.formatter.convert(
                                            std::time::Duration::from_micros(
                                                cur_micros - ent.last_updated,
                                            ),
                                        ))
                                        .weak(),
                                    );

                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        match cached_promise.ready() {
                                            None => {
                                                ui.spinner();
                                            }
                                            Some(Err(_)) => {
                                                let _e = ui.button("\u{1F5D9}");
                                            }
                                            Some(Ok(ent_full)) => {
                                                if ui.button("\u{1F5D0}").clicked() {
                                                    ui.output().copied_text = ent_full.clone();
                                                }
                                            }
                                        }

                                        let resp = ui.add_enabled(
                                            !self.secret.is_empty(),
                                            Button::new("\u{274C}"),
                                        );
                                        let id = resp.id.with("discard_confirmation");
                                        egui::popup::popup_below_widget(ui, id, &resp, |ui| {
                                            ui.set_min_width(80.0);
                                            ui.label("Are you sure?");
                                            ui.horizontal(|ui| {
                                                if ui.button("Yes").clicked() {
                                                    ui.memory().close_popup();
                                                    crate::requests::delete_log(
                                                        &self.base_url,
                                                        ent.id,
                                                        &self.secret,
                                                        ctx.clone(),
                                                    );
                                                }
                                                if ui.button("No").clicked() {
                                                    ui.memory().close_popup();
                                                }
                                            });
                                        });
                                        if resp.clicked() {
                                            ui.memory().open_popup(id);
                                        }

                                        let resp = ui.add_enabled(
                                            !self.secret.is_empty(),
                                            Button::new("\u{2705}"),
                                        );
                                        let id = resp.id.with("keep_confirmation");
                                        egui::popup::popup_below_widget(ui, id, &resp, |ui| {
                                            ui.set_min_width(80.0);
                                            ui.label("Are you sure?");
                                            ui.horizontal(|ui| {
                                                if ui.button("Yes").clicked() {
                                                    ui.memory().close_popup();
                                                    crate::requests::save_log(
                                                        &self.base_url,
                                                        ent.id,
                                                        &self.secret,
                                                        ctx.clone(),
                                                    );
                                                }
                                                if ui.button("No").clicked() {
                                                    ui.memory().close_popup();
                                                }
                                            });
                                        });
                                        if resp.clicked() {
                                            if ent.is_saved {
                                                ui.memory().open_popup(id);
                                            } else {
                                                crate::requests::save_log(
                                                    &self.base_url,
                                                    ent.id,
                                                    &self.secret,
                                                    ctx.clone(),
                                                );
                                            }
                                        }

                                        if ui.button("\u{1F5B9} Open URL").clicked() {
                                            ui.output().open_url = Some(egui::output::OpenUrl {
                                                url: format!("{}/{}", self.base_url, ent.id),
                                                new_tab: true,
                                            });
                                        }
                                    });
                                });
                            })
                            .body(|ui| match cached_promise.ready() {
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
                                    Frame::canvas(ui.style()).show(ui, |ui| {
                                        ui.add(
                                            TextEdit::multiline(&mut ent.trim())
                                                .code_editor()
                                                .desired_width(f32::INFINITY)
                                                .desired_rows(1)
                                                .hint_text("Nothing to see here"),
                                        );
                                    });
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
