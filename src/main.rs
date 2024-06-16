mod ocr_object;
mod translator_object;
mod screen_object;
mod profile_object;
mod area_object;
mod state;
mod window_manager;
mod window;
mod utils;
mod settings;

use adw::prelude::*;
use gtk::{ gio, glib };
use window::Window;

static APP_ID: &str = "org.caioxcezar.game_translator";

fn main() -> glib::ExitCode {
    #[rustfmt::skip]
    gio::resources_register_include!("game_translator.gresource").expect(
        "Failed to register resources."
    );

    // Create a new application
    let app = adw::Application::builder().application_id(APP_ID).build();

    // Connect to signals
    app.connect_startup(|app| {
        setup_shortcuts(app);
        load_css();
    });
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn setup_shortcuts(app: &adw::Application) {
    app.set_accels_for_action("win.save", &["<Ctrl>s"]);
    app.set_accels_for_action("win.configure-page", &["<Ctrl><Shift>t"]);
}

fn build_ui(app: &adw::Application) {
    // Create a new custom window and show it
    let window = Window::new(app);
    window.set_visible(true);
}

fn load_css() {
    // Load the CSS file and add it to the provider
    let provider = gtk::CssProvider::new();
    let path = include_str!("../resources/styles.css");
    provider.load_from_data(path);

    // Add the provider to the default screen
    gtk::style_context_add_provider_for_display(
        &gtk::gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
    );
}
