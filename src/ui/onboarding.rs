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
            Page::OnboardingNewShowKey => Self::onboarding_new_keypair_generated(app, ui),
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

        if ui.button("Create new keypair").clicked() {
            let _ = app.account_manager.generate_keys();
            app.current_page = Page::OnboardingNewShowKey;
        }
    }


    fn onboarding_new_keypair_generated(app: &mut Hoot, ui: &mut egui::Ui) {
        use nostr::ToBech32;
        use crate::keystorage::KeyStorage;

        let first_key = app.account_manager.loaded_keys[0].clone();
        ui.label(format!("New identity: {}", first_key.public_key().to_bech32().unwrap()));

        if ui.button("OK, Save!").clicked() {
            app.account_manager.add_key(&first_key).expect("could not write key");

            app.current_page = Page::Inbox;
        }
    }

    fn onboarding_returning(app: &mut Hoot, ui: &mut egui::Ui) {
        if ui.button("Go Back").clicked() {
            app.current_page = Page::Onboarding;
        }
        ui.label("Welcome Back!");
        ui.text_edit_singleline(&mut app.state.onboarding.secret_input);
    }
}
