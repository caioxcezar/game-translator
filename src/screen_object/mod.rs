mod imp;

use glib::Object;
use gtk::glib;
use image::{ GenericImage, GenericImageView, ImageBuffer, Rgba };
use uuid::Uuid;
use xcap::Window;
use std::cmp;
use crate::{ rect, utils };

glib::wrapper! {
    pub struct ScreenObject(ObjectSubclass<imp::ScreenObject>);
}

impl ScreenObject {
    pub fn new(id: u32, app_name: String, title: String) -> Self {
        Object::builder()
            .property("id", id)
            .property("app-name", app_name)
            .property("title", title)
            .build()
    }
}
#[derive(Default, Clone)]
pub struct ScreenData {
    pub id: u32,
    pub app_name: String,
    pub title: String,
}

impl ScreenData {
    pub fn new(id: u32, app_name: String, title: String) -> ScreenData {
        ScreenData { id, app_name, title }
    }

    pub fn capture_areas(&self, areas: &Vec<rect::Rect>) -> Result<Vec<String>, anyhow::Error> {
        let mut image = self.capture_screen()?;
        let path = utils::temp_path()?;
        let mut strings = vec![];
        for a in areas {
            let copy = image.sub_image(a.x as u32, a.y as u32, a.width as u32, a.height as u32);
            let img_path = format!("{}/{}.png", &path, Uuid::new_v4());
            copy.to_image().save(&img_path)?;
            strings.push(img_path);
        }
        Ok(strings)
    }

    pub fn capture(&self) -> Result<String, anyhow::Error> {
        let image = self.capture_screen()?;
        let path = utils::temp_path()?;
        let path = format!("{}/{}.png", path, Uuid::new_v4());
        image.save(&path)?;
        Ok(path)
    }

    fn capture_screen(&self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, anyhow::Error> {
        let windows = Window::all()?;
        let window = windows.iter().find(|w| w.id() == self.id);
        if window.is_none() {
            return Err(anyhow::anyhow!("Window not found"));
        }
        let window = window.unwrap();
        let monitor = window.current_monitor();
        let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(monitor.width(), monitor.height());
        let other = window.capture_image()?;
        let x = cmp::max(0, window.x());
        let y = cmp::max(0, window.y());
        let width = cmp::min(window.width(), monitor.width());
        let height = cmp::min(window.height(), monitor.height());
        let other = other.view(0, 0, width, height).to_image();
        image.copy_from(&other, x as u32, y as u32)?;
        Ok(image)
    }
}
