#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // for windows release

use eframe::egui::{self, FontDefinitions, Sense, Vec2b};
use egui::FontFamily::Proportional;
use egui::TextStyle::*;
use egui::{Align, FontId, Layout};
use egui_extras::{Column, TableBuilder};
use tracing::{debug, error, info, Level};

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout()); // add log files in prod one day
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(Level::DEBUG)
        .init();

    start_puffin_server();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1024.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Hoot",
        options,
        Box::new(|cc| {
            let _ = &cc.egui_ctx.set_visuals(egui::Visuals::light());
            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert(
                "Inter".to_owned(),
                egui::FontData::from_static(include_bytes!("../fonts/Inter.ttf")),
            );
            fonts
                .families
                .get_mut(&Proportional)
                .unwrap()
                .insert(0, "Inter".to_owned());
            let _ = &cc.egui_ctx.set_fonts(fonts);
            let _ = &cc
                .egui_ctx
                .style_mut(|style| style.visuals.dark_mode = false);
            Box::<Hoot>::default()
        }),
    )
}

#[derive(Debug, PartialEq)]
enum Page {
    Inbox,
    Drafts,
    Starred,
    Archived,
    Trash,
    Post,
}

struct Hoot {
    current_page: Page,
    focused_post: String,
    status: HootStatus,
    nostr: yandk::coordinator::Coordinator,
}

#[derive(Debug, PartialEq)]
enum HootStatus {
    Initalizing,
    Ready,
}

impl Default for Hoot {
    fn default() -> Self {
        Self::new()
    }
}

impl Hoot {
    fn new() -> Self {
        let coordinator = yandk::coordinator::Coordinator::new();

        Self {
            nostr: coordinator,
            current_page: Page::Inbox,
            focused_post: "".into(),
            status: HootStatus::Initalizing,
        }
    }
}

impl eframe::App for Hoot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        match self.status {
            HootStatus::Initalizing => {
                info!("Initalizing Hoot...");
                self.status = HootStatus::Ready;
                let cloned_ctx = ctx.clone();
                let refresh_func = move || {
                    cloned_ctx.request_repaint();
                };
                let _ = self.nostr.add_relay("".to_string(), refresh_func);
            }
            HootStatus::Ready => self.nostr.try_recv(), // we want to recieve events now
        }

        egui::SidePanel::left("sidebar").show(ctx, |ui| {
            ui.heading("Hoot");
            ui.vertical(|ui| {
                ui.style_mut()
                    .text_styles
                    .insert(Button, FontId::new(20.0, Proportional));
                ui.selectable_value(&mut self.current_page, Page::Inbox, "Inbox");
                ui.selectable_value(&mut self.current_page, Page::Drafts, "Drafts");
                ui.selectable_value(&mut self.current_page, Page::Starred, "Starred");
                ui.selectable_value(&mut self.current_page, Page::Archived, "Archived");
                ui.selectable_value(&mut self.current_page, Page::Trash, "Trash");

                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    let my_key = yandk::Pubkey::from_hex(
                        "c5fb6ecc876e0458e3eca9918e370cbcd376901c58460512fe537a46e58c38bb",
                    )
                    .unwrap();
                    let maybe_profile = match self.nostr.get_profile(my_key.bytes()) {
                        Ok(p) => p,
                        Err(e) => {
                            error!("error when getting profile: {}", e);
                            ui.label("Loading...");
                            ui.label("Loading...");
                            None
                        }
                    };
                    if let Some(p) = maybe_profile {
                        let record = p.record();
                        if let Some(profile) = record.profile() {
                            if let Some(nip_05) = profile.nip05() {
                                ui.label(nip_05);
                            } else {
                                ui.label("No Nostr Address");
                            }
                            if let Some(display_name) = profile.display_name() {
                                ui.label(display_name);
                            } else if let Some(name) = profile.name() {
                                ui.label(format!("@{}", name));
                            } else {
                                let hex = my_key.hex();
                                ui.label(hex);
                            }
                        }
                    } else {
                        ui.label("Loading...");
                        ui.label("Loading...");
                    }
                    ui.separator();
                });
            });
        });
        egui::TopBottomPanel::top("search").show(ctx, |ui| {
            ui.heading("Search");
        });
        egui::CentralPanel::default().show(ctx, |ui| match self.current_page {
            Page::Inbox => {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut false, "");
                    ui.heading("Inbox");
                });
                TableBuilder::new(ui)
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .striped(true)
                    .auto_shrink(Vec2b { x: false, y: false })
                    .sense(Sense::click())
                    .header(20.0, |_header| {})
                    .body(|mut body| {
                        puffin::profile_scope!("table rendering");
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut false, "");
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut false, "");
                            });
                            row.col(|ui| {
                                ui.label("Jack Chakany");
                            });
                            row.col(|ui| {
                                ui.label("Hello! Just checking in...");
                            });
                            row.col(|ui| {
                                ui.label("5 min ago");
                            });

                            if row.response().clicked() {
                                self.current_page = Page::Post;
                            }
                        });
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut false, "");
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut false, "");
                            });
                            row.col(|ui| {
                                ui.label("Karnage");
                            });
                            row.col(|ui| {
                                ui.label("New designs!");
                            });
                            row.col(|ui| {
                                ui.label("10 min ago");
                            });

                            if row.response().clicked() {
                                self.current_page = Page::Post;
                            }
                        });
                    });
            }
            Page::Drafts => {
                ui.heading("Drafts");
            }
            Page::Starred => {
                ui.heading("Starred");
            }
            Page::Archived => {
                ui.heading("Archived");
            }
            Page::Trash => {
                ui.heading("Trash");
            }
            Page::Post => {
                // used for viewing messages duh
                ui.heading("Message");
                ui.label(format!("{}", self.focused_post));
            }
        });
    }
}
fn start_puffin_server() {
    puffin::set_scopes_on(true); // tell puffin to collect data

    match puffin_http::Server::new("127.0.0.1:8585") {
        Ok(puffin_server) => {
            debug!("Run: cargo install puffin_viewer && puffin_viewer --url 127.0.0.1:8585");

            std::process::Command::new("puffin_viewer")
                .arg("--url")
                .arg("127.0.0.1:8585")
                .spawn()
                .ok();

            // We can store the server if we want, but in this case we just want
            // it to keep running. Dropping it closes the server, so let's not drop it!
            #[allow(clippy::mem_forget)]
            std::mem::forget(puffin_server);
        }
        Err(err) => {
            error!("Failed to start puffin server: {}", err);
        }
    };
}
