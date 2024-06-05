mod imp;

use glib::Object;
use gtk::glib;
use image::{ GenericImage, GenericImageView, ImageBuffer, Rgba };
use uuid::Uuid;
use xcap::Window;
use std::{ fs, cmp };

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

    pub fn capture_area(
        &self,
        x: u32,
        y: u32,
        width: u32,
        height: u32
    ) -> Result<String, anyhow::Error> {
        let mut image = self.capture_screen()?;
        let copy = image.sub_image(x, y, width, height);
        let path = format!("target/{}.png", Uuid::new_v4());
        copy.to_image().save(&path)?;
        Ok(path)
    }

    pub fn capture(&self) -> Result<String, anyhow::Error> {
        let image = self.capture_screen()?;
        let path = format!("target/{}.png", Uuid::new_v4());
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

    pub fn remove(&self, path: &str) -> Result<(), anyhow::Error> {
        fs::remove_file(path)?;
        Ok(())
    }
}
