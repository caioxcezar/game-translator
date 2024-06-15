mod imp;

use adw::prelude::*;
use adw::subclass::prelude::*;
use glib::Object;
use gtk::{ gio, glib };
use serde::{ Deserialize, Serialize };

use crate::area_object::{ AreaData, AreaObject };

glib::wrapper! {
    pub struct ProfileObject(ObjectSubclass<imp::ProfileObject>);
}

impl ProfileObject {
    pub fn new(
        title: &str,
        app: &str,
        language: &str,
        translation: &str,
        areas: gio::ListStore,
        use_areas: bool
    ) -> Self {
        Object::builder()
            .property("title", title)
            .property("app", app)
            .property("language", language)
            .property("translation", translation)
            .property("areas", areas)
            .property("use-areas", use_areas)
            .build()
    }

    pub fn to_profile_data(&self) -> ProfileData {
        ProfileData {
            title: self.imp().title.borrow().clone(),
            app: self.imp().app.borrow().clone(),
            language: self.imp().language.borrow().clone(),
            translation: self.imp().translation.borrow().clone(),
            use_areas: *self.imp().use_areas.borrow(),
            areas: self
                .areas()
                .iter::<AreaObject>()
                .filter_map(Result::ok)
                .map(|area_object| area_object.area_data())
                .collect(),
        }
    }

    pub fn from_profile_data(profile_data: ProfileData) -> Self {
        let title = profile_data.title;
        let app = profile_data.app;
        let language = profile_data.language;
        let translation = profile_data.translation;
        let use_areas = profile_data.use_areas;
        let areas_to_extend = profile_data.areas
            .into_iter()
            .map(AreaObject::from_area_data)
            .collect::<Vec<AreaObject>>();

        let areas = gio::ListStore::new::<AreaObject>();
        areas.extend_from_slice(&areas_to_extend);

        Self::new(&title, &app, &language, &translation, areas, use_areas)
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ProfileData {
    pub title: String,
    pub app: String,
    pub language: String,
    pub translation: String,
    pub use_areas: bool,
    pub areas: Vec<AreaData>,
}
