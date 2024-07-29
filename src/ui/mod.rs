use eframe::egui;

pub mod compose_window;

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}
