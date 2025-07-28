use anyhow::{Context, Result};
use gtk::glib;
use std::{env, fs, path::PathBuf};

use crate::APP_ID;

pub fn value_in_range(value: i32, min: i32, max: i32) -> bool {
    value >= min && value <= max
}

pub fn split_utf8(text: &str, start: usize, end: usize) -> String {
    text.chars()
        .take(end)
        .skip(start)
        .collect::<String>()
        .trim()
        .to_owned()
}

pub fn temp_path() -> Result<String> {
    let mut temp = env::temp_dir();
    temp.push(APP_ID);
    std::fs::create_dir_all(&temp)?;
    if let Some(path) = temp.to_str() {
        return Ok(path.to_owned());
    }
    Err(anyhow::anyhow!("Failed to get temp path"))
}

pub fn remove_file(path: &str) -> Result<()> {
    fs::remove_file(path)?;
    Ok(())
}

pub fn open_file(path: PathBuf) -> Result<fs::File> {
    fs::File::open(path).ok().context("Failed to open file.")
}

pub fn system_path() -> Result<PathBuf> {
    let mut path = glib::user_data_dir();
    path.push(APP_ID);
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn data_path() -> Result<PathBuf> {
    let mut path = system_path()?;
    path.push("data.json");
    Ok(path)
}

pub fn settings_path() -> Result<PathBuf> {
    let mut path = system_path()?;
    path.push("settings.json");
    Ok(path)
}

pub fn truncate_string(string: &str, size: usize) -> String {
    if string.is_char_boundary(size) {
        format!("{}...", split_utf8(string, 0, size - 3))
    } else {
        string.to_string()
    }
}

pub fn trim_suffix(input: &str, suffix: &str) -> String {
    if let Some(stripped) = input.strip_suffix(suffix) {
        stripped.to_string()
    } else {
        input.to_string()
    }
}
