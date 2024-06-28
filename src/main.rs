#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // for windows release

use eframe::egui::{self, FontDefinitions, Sense, Vec2b};
use egui::FontFamily::Proportional;
use egui::TextStyle::*;
use egui::{Align, FontId, Layout};
use egui_extras::{Column, TableBuilder};
use tracing::{debug, error, info, Level};

mod relay;
mod pool;

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
    relays: pool::RelayPool,
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
        let ctx = ctx.clone();
        let wake_up = move || {
            ctx.request_repaint();
        };
        app.relays.add_url("wss://relay.damus.io".to_string(), wake_up);
        app.status = HootStatus::Ready;
    }

    app.relays.try_recv();
}

fn render_app(ctx: &egui::Context) {
    #[cfg(feature = "profiling")]
    puffin::profile_function!();

    egui::SidePanel::left("Side Navbar").show(ctx, |ui| {
        ui.heading("Hoot");
    });
    egui::TopBottomPanel::top("Search").show(ctx, |ui| {
        ui.heading("Search");
    });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.label("hello there!");
    });
}

impl Hoot {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self {
            current_page: Page::Inbox,
            focused_post: "".into(),
            status: HootStatus::Initalizing,
            relays: pool::RelayPool::new(),
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
            HootStatus::Ready => {}, 
        }

        update_app(self, ctx);
        render_app(ctx);
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
