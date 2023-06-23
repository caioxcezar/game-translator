mod imp;

use adw::subclass::prelude::*;
use adw::{prelude::*, subclass::window};
use gio::Settings;
use glib::{clone, Object};
use gtk::{gio, glib};

use crate::APP_ID;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable,
                    gtk::ConstraintTarget, gtk::Native, gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &adw::Application) -> Self {
        // Create new window
        Object::builder().property("application", app).build()
    }

    fn setup_settings(&self) {
        let settings = Settings::new(APP_ID);
        self.imp()
            .settings
            .set(settings)
            .expect("`settings` should not be set before calling `setup_settings`.");
    }

    fn settings(&self) -> &Settings {
        self.imp()
            .settings
            .get()
            .expect("`settings` should be set in `setup_settings`.")
    }

    fn setup_actions(&self) {
        // let save_action = self.settings().create_action("save");
        // self.add_action(&save_action);

        // let import_action = self.settings().create_action("import");
        // self.add_action(&import_action);
        let action_new_profile = gio::SimpleAction::new("new-profile", None);
        action_new_profile.connect_activate(clone!(@weak self as window => move |_, _| {
            window.navigate("main")
        }));
        self.add_action(&action_new_profile);
    }

    fn navigate(&self, page: &str) {
        self.imp().stack.set_visible_child_name(page);
    }

    fn restore_data(&self) {
        // TODO restore data
        self.navigate("main")
    }
}
