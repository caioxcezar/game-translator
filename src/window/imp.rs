use std::cell::RefCell;

use crate::rect;
use crate::state;
use adw::subclass::prelude::*;
use gio::Settings;
use glib::signal::Inhibit;
use glib::subclass::InitializingObject;
use gtk::{ gio, glib, CompositeTemplate, DrawingArea, DropDown, Stack, CheckButton, Button };
use once_cell::sync::OnceCell;
// ANCHOR: struct
// Object holding the state
#[derive(CompositeTemplate, Default)]
#[template(resource = "/org/caioxcezar/game_translator/window.ui")]
pub struct Window {
    pub settings: OnceCell<Settings>,
    #[template_child]
    pub stack: TemplateChild<Stack>,
    #[template_child]
    pub dd_screen: TemplateChild<DropDown>,
    #[template_child]
    pub dd_ocr: TemplateChild<DropDown>,
    #[template_child]
    pub dd_translation: TemplateChild<DropDown>,
    #[template_child]
    pub chk_full_screen: TemplateChild<CheckButton>,
    #[template_child]
    pub action_button: TemplateChild<Button>,
    #[template_child]
    pub config_button: TemplateChild<Button>,
    pub state: RefCell<state::State>,
    pub drawing_area: DrawingArea,
    pub use_areas: gtk::Switch,
    pub translation_areas: RefCell<Vec<rect::Rect>>,
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
        obj.setup_actions();
        obj.setup_data();
        obj.setup_drag_action();
    }
}
// ANCHOR_END: object_impl

// Trait shared by all widgets
impl WidgetImpl for Window {}

// ANCHOR: window_impl
// Trait shared by all windows
impl WindowImpl for Window {
    fn close_request(&self) -> Inhibit {
        // Pass close request on to the parent
        self.parent_close_request()
    }
}
// ANCHOR_END: window_impl

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}

// Trait shared by all adwaita application windows
impl AdwApplicationWindowImpl for Window {}
