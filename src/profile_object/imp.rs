use std::cell::RefCell;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Properties;
use gtk::{ gio, glib };
use std::cell::OnceCell;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::ProfileObject)]
pub struct ProfileObject {
    #[property(get, set)]
    pub title: RefCell<String>,
    #[property(get, set)]
    pub app: RefCell<String>,
    #[property(get, set)]
    pub language: RefCell<String>,
    #[property(get, set)]
    pub translation: RefCell<String>,
    #[property(get, set)]
    pub use_areas: RefCell<bool>,
    #[property(get, set)]
    pub areas: OnceCell<gio::ListStore>,
}

#[glib::object_subclass]
impl ObjectSubclass for ProfileObject {
    const NAME: &'static str = "ProfileObject";
    type Type = super::ProfileObject;
}

#[glib::derived_properties]
impl ObjectImpl for ProfileObject {}
