#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // for windows release

use eframe::egui::{self, FontDefinitions, Sense, Vec2b};
use egui::FontFamily::Proportional;
use egui::TextStyle::*;
use egui::{Align, FontId, Layout};
use egui_extras::{Column, TableBuilder};
use tracing::{debug, error, info, Level};

mod account_manager;
mod error;
mod keystorage;
mod relay;
mod ui;

fn main() -> Result<(), eframe::Error> {
    let (non_blocking, _guard) = tracing_appender::non_blocking(std::io::stdout()); // add log files in prod one day
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_max_level(Level::DEBUG)
        .init();

    #[cfg(feature = "profiling")]
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
            Box::new(Hoot::new(cc))
        }),
    )
}

#[derive(Debug, PartialEq)]
pub enum Page {
    Inbox,
    Drafts,
    Settings,
    // TODO: fix this mess
    Onboarding,
    OnboardingNew,
    OnboardingNewShowKey,
    OnboardingReturning,
}

// for storing the state of different components and such.
#[derive(Default)]
pub struct HootState {
    pub onboarding: ui::onboarding::OnboardingState,
}

pub struct Hoot {
    pub page: Page,
    focused_post: String,
    status: HootStatus,
    state: HootState,
    relays: relay::RelayPool,
    ndb: nostrdb::Ndb,
    events: Vec<nostr::Event>,
    pub windows: Vec<Box<ui::compose_window::ComposeWindow>>,
    account_manager: account_manager::AccountManager,
}

#[derive(Debug, PartialEq)]
enum HootStatus {
    Initalizing,
    Ready,
}

fn update_app(app: &mut Hoot, ctx: &egui::Context) {
    #[cfg(feature = "profiling")]
    puffin::profile_function!();

    if app.status == HootStatus::Initalizing {
        info!("Initalizing Hoot...");
        let ctx = ctx.clone();
        let wake_up = move || {
            ctx.request_repaint();
        };
        app.relays
            .add_url("wss://relay.damus.io".to_string(), wake_up.clone());
        app.relays
            .add_url("wss://relay-dev.hoot.sh".to_string(), wake_up);
        app.status = HootStatus::Ready;
        info!("Hoot Ready");
    }

    let new_val = app.relays.try_recv();
    if new_val.is_some() {
        info!("{:?}", new_val.clone());

        use relay::RelayMessage;
        let deserialized: RelayMessage =
            serde_json::from_str(new_val.unwrap().as_str()).expect("relay sent us bad json");

        use RelayMessage::*;
        match deserialized {
            Event {
                subscription_id,
                event,
            } => {
                app.events.push(event);
            }
            _ => {
                // who cares rn
            }
        }
    }
}

fn render_app(app: &mut Hoot, ctx: &egui::Context) {
    #[cfg(feature = "profiling")]
    puffin::profile_function!();

    if app.page == Page::Onboarding
        || app.page == Page::OnboardingNew
        || app.page == Page::OnboardingNewShowKey
        || app.page == Page::OnboardingReturning
    {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui::onboarding::OnboardingScreen::ui(app, ui);
        });
    } else {
        egui::SidePanel::left("Side Navbar").show(ctx, |ui| {
            ui.heading("Hoot");
            if ui.button("Inbox").clicked() {
                app.page = Page::Inbox;
            }
            if ui.button("Drafts").clicked() {
                app.page = Page::Drafts;
            }
            if ui.button("Settings").clicked() {
                app.page = Page::Settings;
            }
        });

        egui::TopBottomPanel::top("Search").show(ctx, |ui| {
            ui.heading("Search");
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // todo: fix
            for window in &mut app.windows {
                window.show(ui);
            }

            if app.page == Page::Inbox {
                ui.label("hello there!");
                if ui.button("Compose").clicked() {
                    let mut new_window = Box::new(ui::compose_window::ComposeWindow::new());
                    new_window.show(ui);
                    app.windows.push(new_window);
                }

                if ui.button("Send Test Event").clicked() {
                    let temp_keys = nostr::Keys::generate();
                    // todo: lmao
                    let new_event = nostr::EventBuilder::text_note("GFY!", [])
                        .to_event(&temp_keys)
                        .unwrap();
                    let event_json = crate::relay::ClientMessage::Event { event: new_event };
                    let _ = &app
                        .relays
                        .send(ewebsock::WsMessage::Text(
                            serde_json::to_string(&event_json).unwrap(),
                        ))
                        .unwrap();
                }

                if ui.button("Get kind 1 notes").clicked() {
                    let mut filter = nostr::types::Filter::new();
                    filter = filter.kind(nostr::Kind::TextNote);
                    let mut sub = crate::relay::Subscription::default();
                    sub.filter(filter);
                    let c_msg = crate::relay::ClientMessage::from(sub);

                    let _ = &app
                        .relays
                        .send(ewebsock::WsMessage::Text(
                            serde_json::to_string(&c_msg).unwrap(),
                        ))
                        .unwrap();
                }

                TableBuilder::new(ui)
                    .column(Column::auto())
                    .column(Column::auto())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .column(Column::remainder())
                    .striped(true)
                    .sense(Sense::click())
                    .auto_shrink(Vec2b { x: false, y: false })
                    .header(20.0, |_header| {})
                    .body(|mut body| {
                        for event in app.events.clone() {
                            body.row(30.0, |mut row| {
                                row.col(|ui| {
                                    ui.checkbox(&mut false, "");
                                });
                                row.col(|ui| {
                                    ui.checkbox(&mut false, "");
                                });
                                row.col(|ui| {
                                    ui.label(event.pubkey.to_string());
                                });
                                row.col(|ui| {
                                    ui.label(event.content.clone());
                                });
                                row.col(|ui| {
                                    ui.label("2 minutes ago");
                                });
                            });
                        }
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.checkbox(&mut false, "");
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut false, "");
                            });
                            row.col(|ui| {
                                ui.label("Elon Musk");
                            });
                            row.col(|ui| {
                                ui.label("Second Test Message");
                            });
                            row.col(|ui| {
                                ui.label("2 minutes ago");
                            });
                        });

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
                                ui.label("Message Content");
                            });
                            row.col(|ui| {
                                ui.label("5 minutes ago");
                            });
                        });
                    });
            } else if app.page == Page::Settings {
                ui.label("Settings");
                ui.label(format!(
                    "Connected Relays: {}",
                    &app.relays.get_number_of_relays()
                ));

                if ui.button("fetch keys").clicked() {
                    let _ = app.account_manager.load_keys();
                }

                ui.vertical(|ui| {
                    use nostr::ToBech32;
                    for key in app.account_manager.loaded_keys.clone() {
                        ui.label(format!("Key ID: {}", key.public_key().to_bech32().unwrap()));
                    }
                });
            }
        });
    }
}

impl Hoot {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let storage_dir = eframe::storage_dir("Hoot").unwrap();
        let mut ndb_config = nostrdb::Config::new();
        ndb_config.set_ingester_threads(3);

        let ndb = nostrdb::Ndb::new(storage_dir.to_str().unwrap(), &ndb_config)
            .expect("could not load nostrdb");
        Self {
            page: Page::Inbox,
            focused_post: "".into(),
            status: HootStatus::Initalizing,
            state: Default::default(),
            relays: relay::RelayPool::new(),
            ndb,
            events: Vec::new(),
            windows: Vec::new(),
            account_manager: account_manager::AccountManager::new(),
        }
    }
}

impl eframe::App for Hoot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        update_app(self, ctx);
        render_app(self, ctx);
    }
}

#[cfg(feature = "profiling")]
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
