#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // for windows release

use eframe::egui::{self, FontDefinitions, Vec2b};
use egui::FontFamily::Proportional;
use egui::TextStyle::*;
use egui::{Align, FontId, Layout};
use egui_extras::{Column, TableBuilder};

fn main() -> Result<(), eframe::Error> {
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

#[derive(PartialEq)]
enum Page {
    Inbox,
    Drafts,
    Starred,
    Archived,
    Trash,
}

struct Hoot {
    current_page: Page,
}

impl Default for Hoot {
    fn default() -> Self {
        Self {
            current_page: Page::Inbox,
        }
    }
}

impl eframe::App for Hoot {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    .column(Column::auto())
                    .striped(true)
                    .auto_shrink(Vec2b { x: false, y: false })
                    .header(20.0, |mut header| {})
                    .body(|mut body| {
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
        });
    }
}
