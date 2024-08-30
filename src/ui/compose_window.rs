use crate::Hoot;
use eframe::egui::{self, RichText};
use rand::random;

pub struct ComposeWindow {
    title: Option<RichText>,
    id: egui::Id,
    subject: String,
    to_field: String,
    content: String,
}

impl ComposeWindow {
    pub fn new() -> Self {
        Self {
            title: None,
            id: egui::Id::new(random::<u32>()),
            subject: String::from("New Message"),
            to_field: String::new(),
            content: String::new(),
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::Window::new(&self.subject)
            .id(self.id)
            .show(ui.ctx(), |ui| {
                ui.label("Hello!");
                ui.vertical(|ui| {
                    ui.text_edit_singleline(&mut self.to_field);
                    ui.text_edit_singleline(&mut self.subject);
                    ui.add_sized(
                        ui.available_size(),
                        egui::TextEdit::multiline(&mut self.content),
                    );
                });
            });
    }
}
