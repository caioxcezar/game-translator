use std::{ env, fs, str::Lines };

pub fn value_in_range(value: i32, min: i32, max: i32) -> bool {
    value >= min && value <= max
}

pub fn split_utf8(text: &str, start: usize, end: usize) -> String {
    text.chars().take(end).skip(start).collect::<String>().trim().to_owned()
}

pub fn temp_path() -> Result<String, anyhow::Error> {
    let temp = env::temp_dir().to_str().unwrap().to_string();
    let path = format!("{}game_translator", temp);
    fs::DirBuilder::new().recursive(true).create(&path)?;
    Ok(path)
}

pub fn remove_file(path: &str) -> Result<(), anyhow::Error> {
    fs::remove_file(path)?;
    Ok(())
}

pub fn calc_font_size(lines: &Lines, width: f64) -> f64 {
    let max_str = lines.clone().max().unwrap().split("").count();
    width / (max_str as f64)
}
