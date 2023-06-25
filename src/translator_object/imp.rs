use std::cell::RefCell;

use glib::{ParamSpec, Properties, Value};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use super::TranslatorData;

#[derive(Properties, Default)]
#[properties(wrapper_type = super::TranslatorObject)]
pub struct TranslatorObject {
    #[property(name = "code", get, set, type = String, member = code)]
    #[property(name = "language", get, set, type = String, member = language)]
    pub data: RefCell<TranslatorData>,
}

#[glib::object_subclass]
impl ObjectSubclass for TranslatorObject {
    const NAME: &'static str = "TranslatorObject";
    type Type = super::TranslatorObject;
}

impl ObjectImpl for TranslatorObject {
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
