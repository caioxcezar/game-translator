use std::cell::RefCell;

use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::AreaData;

// Object holding the state
#[derive(Properties, Default)]
#[properties(wrapper_type = super::AreaObject)]
pub struct AreaObject {
    #[property(name = "x", get, set, type = i32, member = x)]
    #[property(name = "y", get, set, type = i32, member = y)]
    #[property(name = "width", get, set, type = i32, member = width)]
    #[property(name = "height", get, set, type = i32, member = height)]
    #[property(name = "text", get, set, type = String, member = text)]
    pub data: RefCell<AreaData>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for AreaObject {
    const NAME: &'static str = "AreaObject";
    type Type = super::AreaObject;
}

// Trait shared by all GObjects
#[glib::derived_properties]
impl ObjectImpl for AreaObject {}
