use std::cell::RefCell;

use crate::{ profile_object::{ ProfileData, ProfileObject }, settings::Settings, state, utils };
use adw::subclass::prelude::*;
use glib::subclass::InitializingObject;
use gtk::{ gio, glib, CompositeTemplate, prelude::ListModelExtManual };
use headless_chrome::Browser;
use once_cell::sync::OnceCell;
use std::fs;
// ANCHOR: struct
// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/caioxcezar/game_translator/window.ui")]
pub struct Window {
    pub settings: RefCell<Settings>,
    #[template_child]
    pub stack: TemplateChild<gtk::Stack>,
    #[template_child]
    pub dd_screen: TemplateChild<gtk::DropDown>,
    #[template_child]
    pub dd_ocr: TemplateChild<gtk::DropDown>,
    #[template_child]
    pub dd_translation: TemplateChild<gtk::DropDown>,
    #[template_child]
    pub chk_full_screen: TemplateChild<gtk::CheckButton>,
    #[template_child]
    pub action_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub title: TemplateChild<gtk::Entry>,
    #[template_child]
    pub config_button: TemplateChild<gtk::Button>,
    #[template_child]
    pub profiles_list: TemplateChild<gtk::ListBox>,
    pub profiles: OnceCell<gio::ListStore>,
    pub running: RefCell<bool>,
    pub browser: OnceCell<Browser>,
    pub state: RefCell<state::State>,
    pub drawing_area: gtk::DrawingArea,
    pub use_areas: gtk::Switch,
}
// ANCHOR_END: struct

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    // `NAME` needs to match `class` attribute of template
    const NAME: &'static str = "ProfileWindow";
    type Type = super::Window;
    type ParentType = adw::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// ANCHOR: object_impl
// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        // Call "constructed" on parent
        self.parent_constructed();
        // Setup
        let obj = self.obj();
        obj.setup_settings();
        obj.setup_data();
        obj.setup_actions();
        obj.setup_drag_action();
        obj.setup_browser();
        obj.setup_profiles();
        let _ = obj.restore_data();
    }
}
// ANCHOR_END: object_impl

// Trait shared by all widgets
impl WidgetImpl for Window {}

// ANCHOR: window_impl
// Trait shared by all windows
impl WindowImpl for Window {
    fn close_request(&self) -> glib::Propagation {
        if let Ok(path) = utils::temp_path() {
            fs::remove_dir_all(path).expect("Could not remove temp files");
        }

        if let Ok(path) = utils::data_path() {
            let backup_data = self
                .obj()
                .profiles()
                .iter::<ProfileObject>()
                .filter_map(|collection_object| collection_object.ok())
                .map(|collection_object| collection_object.to_profile_data())
                .collect::<Vec<ProfileData>>();

            let file = fs::File::create(path).expect("Could not create json file");
            serde_json::to_writer(file, &backup_data).expect("Could not write data to json file");
        }

        let _ = self.obj().settings().update_json();

        self.parent_close_request()
    }
}
// ANCHOR_END: window_impl

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}

// Trait shared by all adwaita application windows
impl AdwApplicationWindowImpl for Window {}
