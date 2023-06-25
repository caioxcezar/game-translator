mod ocr_object;
mod translator_object;
mod window;
use adw::prelude::*;
use gtk::{gio, glib};
use window::Window;

static APP_ID: &str = "org.caioxcezar.game_translator";

fn main() -> glib::ExitCode {
    gio::resources_register_include!("game_translator.gresource")
        .expect("Failed to register resources.");

    // Create a new application
    let app = adw::Application::builder().application_id(APP_ID).build();

    // Connect to signals
    app.connect_startup(setup_shortcuts);
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn setup_shortcuts(app: &adw::Application) {
    app.set_accels_for_action("win.save", &["<Ctrl>s"]);
}

fn build_ui(app: &adw::Application) {
    // Create a new custom window and show it
    let window = Window::new(app);
    window.set_visible(true);
}
