use eframe::egui;

pub mod compose_window;
pub mod onboarding;
pub mod settings;

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}
