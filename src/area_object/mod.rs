mod imp;

use adw::subclass::prelude::*;
use glib::Object;
use gtk::glib;
use serde::{ Deserialize, Serialize };

glib::wrapper! {
    pub struct AreaObject(ObjectSubclass<imp::AreaObject>);
}

impl AreaObject {
    pub fn new(x: i32, y: i32, width: i32, height: i32, text: String) -> Self {
        Object::builder()
            .property("x", x)
            .property("y", y)
            .property("width", width)
            .property("height", height)
            .property("text", text)
            .build()
    }

    pub fn area_data(&self) -> AreaData {
        self.imp().data.borrow().clone()
    }

    pub fn from_area_data(area_data: AreaData) -> Self {
        Self::new(area_data.x, area_data.y, area_data.width, area_data.height, area_data.text)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AreaData {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    #[serde(skip_serializing, default)]
    pub text: String,
}
