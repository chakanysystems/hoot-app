#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // for windows release

use std::time::SystemTime;

use eframe::egui::{self, FontDefinitions, Sense, Vec2b};
use egui::FontFamily::Proportional;
use egui::TextStyle::*;
use egui::{Align, FontId, Layout};
use egui_extras::{Column, TableBuilder};
use std::thread;
use tokio::runtime;
use tracing::{debug, error, info, Level};

fn main() -> Result<(), eframe::Error> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout()); // add log files in prod one day
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(Level::DEBUG)
        .init();
    thread::spawn(move || {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                tokio::task::spawn(async {
                    let mut relay =
                        yandk::relay::Relay::new("wss://relay.damus.io".to_string()).unwrap();
                    match relay.connect().await {
                        Ok(g) => g,
                        Err(e) => error!("error, {:?}", e),
                    }
                    let mut relay2 = yandk::relay::Relay::new("wss://nos.lol".to_string()).unwrap();
                    match relay2.connect().await {
                        Ok(g) => g,
                        Err(e) => error!("error, {:?}", e),
                    }
                });
                loop {
                    std::thread::yield_now();
                }
            });
    });

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
}

#[derive(Debug, PartialEq)]
enum HootStatus {
    Initalizing,
    Ready,
}

impl Default for Hoot {
    fn default() -> Self {
        Self {
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
            }
            HootStatus::Ready => {}
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
                    ui.label("jack@chakany.systems");
                    ui.label("Jack Chakany");
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
