use termion::color::*;

pub fn font_red(text: &str) -> String {
    format!("{}{text}{}", Fg(termion::color::Red), font_reset())
}

pub fn font_blue(text: &str) -> String {
    format!("{}{text}{}", Fg(Blue), font_reset())
}

pub fn font_reset() -> String {
    format!("{}{}", Reset.bg_str(), Reset.fg_str())
}
