use std::{ env, fs, path::PathBuf, str::Lines };
use anyhow::Context;
use gtk::glib;

use crate::APP_ID;

pub fn value_in_range(value: i32, min: i32, max: i32) -> bool {
    value >= min && value <= max
}

pub fn split_utf8(text: &str, start: usize, end: usize) -> String {
    text.chars().take(end).skip(start).collect::<String>().trim().to_owned()
}

pub fn temp_path() -> Result<String, anyhow::Error> {
    let mut temp = env::temp_dir();
    temp.push(APP_ID);
    std::fs::create_dir_all(&temp)?;
    if let Some(path) = temp.to_str() {
        return Ok(path.to_owned());
    }
    Err(anyhow::anyhow!("Failed to get temp path"))
}

pub fn remove_file(path: &str) -> Result<(), anyhow::Error> {
    fs::remove_file(path)?;
    Ok(())
}

pub fn open_file(path: PathBuf) -> Result<fs::File, anyhow::Error> {
    fs::File::open(path).ok().context("Failed to open file.")
}

pub fn calc_font_size(lines: &Lines, width: i32, height: i32) -> f64 {
    let (width, height) = (width as f64, height as f64);
    let line_count = lines.clone().count() as f64;
    let char_count = lines.clone().max().unwrap().split("").count() as f64;
    let height = height / line_count;
    let width = width / char_count;
    let area = width * height;
    area.sqrt()
}

pub fn data_path() -> Result<PathBuf, anyhow::Error> {
    let mut path = glib::user_data_dir();
    path.push(APP_ID);
    std::fs::create_dir_all(&path)?;
    path.push("data.json");
    Ok(path)
}

pub fn settings_path() -> Result<PathBuf, anyhow::Error> {
    let mut path = glib::user_data_dir();
    path.push(APP_ID);
    std::fs::create_dir_all(&path)?;
    path.push("settings.json");
    Ok(path)
}
