use std::fs;
use image::{ GenericImage, ImageBuffer, Rgba };
use uuid::Uuid;
use xcap::Window;

#[derive(Default, Clone)]
pub struct Screen {
    pub id: usize,
}

impl Screen {
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
        let window = windows.iter().find(|w| w.title().eq("WinLister"));
        if window.is_none() {
            return Err(anyhow::anyhow!("Window not found"));
        }
        let window = window.unwrap();
        let monitor = window.current_monitor();
        let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(monitor.width(), monitor.height());
        let other = window.capture_image()?;
        let x = if window.x() < 0 { 0 } else { window.x() as u32 };
        image.copy_from(&other, x, window.y() as u32)?;
        Ok(image)
    }
    pub fn remove(&self, path: &str) -> Result<(), anyhow::Error> {
        fs::remove_file(path)?;
        Ok(())
    }
}
