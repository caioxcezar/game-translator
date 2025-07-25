use std::cell::RefCell;

use glib::{ParamSpec, Properties, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::ScreenData;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::ScreenObject)]
pub struct ScreenObject {
    #[property(name = "id", get, set, type = u32, member = id)]
    #[property(name = "app-name", get, set, type = String, member = app_name)]
    #[property(name = "title", get, set, type = String, member = title)]
    pub data: RefCell<ScreenData>,
}

#[glib::object_subclass]
impl ObjectSubclass for ScreenObject {
    const NAME: &'static str = "ScreenObject";
    type Type = super::ScreenObject;
}

impl ObjectImpl for ScreenObject {
    fn properties() -> &'static [ParamSpec] {
        Self::derived_properties()
    }

    fn set_property(&self, id: usize, value: &Value, pspec: &ParamSpec) {
        self.derived_set_property(id, value, pspec)
    }

    fn property(&self, id: usize, pspec: &ParamSpec) -> Value {
        self.derived_property(id, pspec)
    }
}
