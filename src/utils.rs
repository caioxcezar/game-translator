pub fn value_in_range(value: i32, min: i32, max: i32) -> bool {
    value >= min && value <= max
}

pub fn split_utf8(text: &str, start: usize, end: usize) -> String {
    text.chars().take(end).skip(start).collect::<String>().trim().to_owned()
}
