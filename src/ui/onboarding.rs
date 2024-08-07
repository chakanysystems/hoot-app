use crate::{Hoot, Page};
use eframe::egui;
use tracing::error;

#[derive(Default)]
pub struct OnboardingState {
    // for nsecs, etc.
    pub secret_input: String,
}

pub struct OnboardingScreen {}

impl OnboardingScreen {
    pub fn ui(app: &mut Hoot, ui: &mut egui::Ui) {
        ui.heading("Welcome to Hoot Mail!");

        match app.current_page {
            Page::Onboarding => Self::onboarding_home(app, ui),
            Page::OnboardingNew => Self::onboarding_new(app, ui),
            Page::OnboardingReturning => Self::onboarding_returning(app, ui),
            _ => error!("OnboardingScreen should not be displayed when page is not Onboarding!"),
        }
    }

    fn onboarding_home(app: &mut Hoot, ui: &mut egui::Ui) {
        if ui.button("I am new to Hoot Mail").clicked() {
            app.current_page = Page::OnboardingNew;
        }

        if ui.button("I have used Hoot Mail before.").clicked() {
            app.current_page = Page::OnboardingReturning;
        }
    }

    fn onboarding_new(app: &mut Hoot, ui: &mut egui::Ui) {
        if ui.button("Go Back").clicked() {
            app.current_page = Page::Onboarding;
        }
        ui.label("To setup Hoot Mail, you need a nostr identity.");
    }

    fn onboarding_returning(app: &mut Hoot, ui: &mut egui::Ui) {
        if ui.button("Go Back").clicked() {
            app.current_page = Page::Onboarding;
        }
        ui.label("Welcome Back!");
        ui.text_edit_singleline(&mut app.state.onboarding.secret_input);
    }
}
