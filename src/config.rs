use eframe::egui::Color32;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeekStart {
    Sunday,
    Monday,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub app_id: String,
    pub close_on_unfocus: bool,
    pub week_start: WeekStart,
    pub show_week_numbers: bool,

    pub font_family: String,
    pub font_weight: u16,
    pub bold_weight: u16,
    pub font_size: f32,
    pub header_font_size: f32,
    pub weekday_font_size: f32,

    pub padding: f32,
    pub cell_width: f32,
    pub cell_height: f32,
    pub header_height: f32,
    pub header_gap: f32,
    pub today_radius: f32,
    pub hover_radius: f32,

    pub colors: Colors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Colors {
    pub background: String,
    pub header_background: String,
    pub header_border: String,
    pub header_text: String,
    pub arrow: String,
    pub arrow_hover: String,
    pub weekday_background: String,
    pub weekday_text: String,
    pub text: String,
    pub weekend_text: String,
    pub other_month_text: String,
    pub week_number_text: String,
    pub today_background: String,
    pub today_text: String,
    pub selected_border: String,
    pub hover: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            app_id: "simplecal".into(),
            close_on_unfocus: true,
            week_start: WeekStart::Sunday,
            show_week_numbers: false,
            font_family: "JetBrainsMono Nerd Font".into(),
            font_weight: 300,
            bold_weight: 700,
            font_size: 14.0,
            header_font_size: 14.0,
            weekday_font_size: 14.0,
            padding: 5.0,
            cell_width: 40.0,
            cell_height: 32.0,
            header_height: 26.0,
            header_gap: 4.0,
            today_radius: 0.0,
            hover_radius: 3.0,
            colors: Colors::default(),
        }
    }
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            background: "#3b4252".into(),
            header_background: "#3b4252".into(),
            header_border: "#2e3440".into(),
            header_text: "#e5e9f0".into(),
            arrow: "#a3abb9".into(),
            arrow_hover: "#eceff4".into(),
            weekday_background: "#5e81ac".into(),
            weekday_text: "#eceff4".into(),
            text: "#eceff4".into(),
            weekend_text: "#eceff4".into(),
            other_month_text: "#7b8394".into(),
            week_number_text: "#81a1c1".into(),
            today_background: "#5e81ac".into(),
            today_text: "#ffffff".into(),
            selected_border: "#81a1c1".into(),
            hover: "#ffffff1a".into(),
        }
    }
}

pub struct Palette {
    pub background: Color32,
    pub header_background: Color32,
    pub header_border: Color32,
    pub header_text: Color32,
    pub arrow: Color32,
    pub arrow_hover: Color32,
    pub weekday_background: Color32,
    pub weekday_text: Color32,
    pub text: Color32,
    pub weekend_text: Color32,
    pub other_month_text: Color32,
    pub week_number_text: Color32,
    pub today_background: Color32,
    pub today_text: Color32,
    pub selected_border: Color32,
    pub hover: Color32,
}

impl Colors {
    pub fn palette(&self) -> Palette {
        let d = Colors::default();
        let p = |v: &str, fallback: &str, name: &str| {
            parse_color(v).unwrap_or_else(|| {
                eprintln!("simplecal: invalid color for {name}: {v:?}");
                parse_color(fallback).unwrap()
            })
        };
        Palette {
            background: p(&self.background, &d.background, "background"),
            header_background: p(&self.header_background, &d.header_background, "header_background"),
            header_border: p(&self.header_border, &d.header_border, "header_border"),
            header_text: p(&self.header_text, &d.header_text, "header_text"),
            arrow: p(&self.arrow, &d.arrow, "arrow"),
            arrow_hover: p(&self.arrow_hover, &d.arrow_hover, "arrow_hover"),
            weekday_background: p(&self.weekday_background, &d.weekday_background, "weekday_background"),
            weekday_text: p(&self.weekday_text, &d.weekday_text, "weekday_text"),
            text: p(&self.text, &d.text, "text"),
            weekend_text: p(&self.weekend_text, &d.weekend_text, "weekend_text"),
            other_month_text: p(&self.other_month_text, &d.other_month_text, "other_month_text"),
            week_number_text: p(&self.week_number_text, &d.week_number_text, "week_number_text"),
            today_background: p(&self.today_background, &d.today_background, "today_background"),
            today_text: p(&self.today_text, &d.today_text, "today_text"),
            selected_border: p(&self.selected_border, &d.selected_border, "selected_border"),
            hover: p(&self.hover, &d.hover, "hover"),
        }
    }
}

pub fn parse_color(s: &str) -> Option<Color32> {
    let h = s.trim().strip_prefix('#')?;
    let digits: Vec<u8> = h
        .chars()
        .map(|c| c.to_digit(16).map(|d| d as u8))
        .collect::<Option<_>>()?;
    let pair = |i: usize| digits[i] * 16 + digits[i + 1];
    let (r, g, b, a) = match digits.len() {
        3 => (digits[0] * 17, digits[1] * 17, digits[2] * 17, 255),
        4 => (digits[0] * 17, digits[1] * 17, digits[2] * 17, digits[3] * 17),
        6 => (pair(0), pair(2), pair(4), 255),
        8 => (pair(0), pair(2), pair(4), pair(6)),
        _ => return None,
    };
    Some(Color32::from_rgba_unmultiplied(r, g, b, a))
}

pub fn default_path() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"));
    base.join("simplecal").join("config.toml")
}

pub fn load(path: &Path) -> Config {
    match std::fs::read_to_string(path) {
        Ok(text) => toml::from_str(&text).unwrap_or_else(|e| {
            eprintln!("simplecal: {}: {e}", path.display());
            Config::default()
        }),
        Err(_) => Config::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_colors() {
        assert_eq!(parse_color("#fff"), Some(Color32::WHITE));
        assert_eq!(parse_color("#5e81ac"), Some(Color32::from_rgb(0x5e, 0x81, 0xac)));
        assert_eq!(parse_color("#ffffff1a"), Some(Color32::from_rgba_unmultiplied(255, 255, 255, 0x1a)));
        assert_eq!(parse_color("5e81ac"), None);
        assert_eq!(parse_color("#12345"), None);
        assert_eq!(parse_color("#zzzzzz"), None);
    }

    #[test]
    fn example_config_matches_defaults() {
        let example: Config = toml::from_str(include_str!("../config.example.toml")).unwrap();
        assert_eq!(toml::to_string(&example).unwrap(), toml::to_string(&Config::default()).unwrap());
    }

    #[test]
    fn partial_config_keeps_defaults() {
        let cfg: Config = toml::from_str("week_start = \"monday\"\n[colors]\ntext = \"#000\"").unwrap();
        assert_eq!(cfg.week_start, WeekStart::Monday);
        assert_eq!(cfg.colors.text, "#000");
        assert_eq!(cfg.colors.background, Colors::default().background);
    }
}
