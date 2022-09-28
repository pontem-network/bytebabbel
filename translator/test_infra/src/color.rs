use termion::color::{Blue, Fg, Green, Red, Reset as ResetColor, Yellow};
use termion::style::{Bold, Reset as RESET_STYLE};

pub fn font_red(text: &str) -> String {
    format!("{}{text}{}", Fg(Red), font_reset())
}

pub fn font_green(text: &str) -> String {
    format!("{}{text}{}", Fg(Green), font_reset())
}

pub fn font_blue(text: &str) -> String {
    format!("{}{text}{}", Fg(Blue), font_reset())
}

pub fn font_reset() -> String {
    format!("{}{}", ResetColor.bg_str(), ResetColor.fg_str())
}

pub fn font_yellow(text: &str) -> String {
    format!("{}{text}{}", Fg(Yellow), font_reset())
}

pub fn bold(text: &str) -> String {
    format!("{}{text}{}", Bold, RESET_STYLE)
}
