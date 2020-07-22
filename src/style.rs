use crate::config::{ConfigColor, ConfigColorByTheme, ConfigColumnStyle, ConfigStyle, ConfigTheme};
use ansi_term::{
    ANSIGenericString,
    Color::{self, Fixed},
    Style,
};
use lazy_static::lazy_static;

lazy_static! {
    static ref BRIGHT_BLACK: Style = Fixed(8).normal();
    static ref BRIGHT_RED: Style = Fixed(9).normal();
    static ref BRIGHT_GREEN: Style = Fixed(10).normal();
    static ref BRIGHT_YELLOW: Style = Fixed(11).normal();
    static ref BRIGHT_BLUE: Style = Fixed(12).normal();
    static ref BRIGHT_MAGENTA: Style = Fixed(13).normal();
    static ref BRIGHT_CYAN: Style = Fixed(14).normal();
    static ref BRIGHT_WHITE: Style = Fixed(15).normal();
    static ref BLACK: Style = Fixed(0).normal();
    static ref RED: Style = Fixed(1).normal();
    static ref GREEN: Style = Fixed(2).normal();
    static ref YELLOW: Style = Fixed(3).normal();
    static ref BLUE: Style = Fixed(4).normal();
    static ref MAGENTA: Style = Fixed(5).normal();
    static ref CYAN: Style = Fixed(6).normal();
    static ref WHITE: Style = Fixed(7).normal();
}

fn apply_style_by_state<'a>(
    x: String,
    s: &ConfigStyle,
    theme: &ConfigTheme,
    faded: bool,
) -> ANSIGenericString<'a, str> {
    match x {
        ref x if x.contains('D') => apply_color(x.to_string(), &s.by_state.color_d, theme, faded),
        ref x if x.contains('R') => apply_color(x.to_string(), &s.by_state.color_r, theme, faded),
        ref x if x.contains('S') => apply_color(x.to_string(), &s.by_state.color_s, theme, faded),
        ref x if x.contains('T') => apply_color(x.to_string(), &s.by_state.color_t, theme, faded),
        ref x if x.contains('t') => apply_color(x.to_string(), &s.by_state.color_t, theme, faded),
        ref x if x.contains('Z') => apply_color(x.to_string(), &s.by_state.color_z, theme, faded),
        ref x if x.contains('X') => apply_color(x.to_string(), &s.by_state.color_x, theme, faded),
        ref x if x.contains('K') => apply_color(x.to_string(), &s.by_state.color_k, theme, faded),
        ref x if x.contains('W') => apply_color(x.to_string(), &s.by_state.color_w, theme, faded),
        ref x if x.contains('P') => apply_color(x.to_string(), &s.by_state.color_p, theme, faded),
        _ => apply_color(x, &s.by_state.color_x, theme, faded),
    }
}

fn apply_style_by_unit<'a>(
    x: String,
    s: &ConfigStyle,
    theme: &ConfigTheme,
    faded: bool,
) -> ANSIGenericString<'a, str> {
    match x {
        ref x if x.contains('K') => apply_color(x.to_string(), &s.by_unit.color_k, theme, faded),
        ref x if x.contains('M') => apply_color(x.to_string(), &s.by_unit.color_m, theme, faded),
        ref x if x.contains('G') => apply_color(x.to_string(), &s.by_unit.color_g, theme, faded),
        ref x if x.contains('T') => apply_color(x.to_string(), &s.by_unit.color_t, theme, faded),
        ref x if x.contains('P') => apply_color(x.to_string(), &s.by_unit.color_p, theme, faded),
        _ => apply_color(x, &s.by_unit.color_x, theme, faded),
    }
}

fn apply_style_by_percentage<'a>(
    x: String,
    s: &ConfigStyle,
    theme: &ConfigTheme,
    faded: bool,
) -> ANSIGenericString<'a, str> {
    let value: f64 = x.trim().parse().unwrap_or(0.0);
    if value > 100.0 {
        apply_color(x, &s.by_percentage.color_100, theme, faded)
    } else if value > 75.0 {
        apply_color(x, &s.by_percentage.color_075, theme, faded)
    } else if value > 50.0 {
        apply_color(x, &s.by_percentage.color_050, theme, faded)
    } else if value > 25.0 {
        apply_color(x, &s.by_percentage.color_025, theme, faded)
    } else {
        apply_color(x, &s.by_percentage.color_000, theme, faded)
    }
}

fn hexcode2color(hexcode: &str) -> Style {
    let str_r: &str = hexcode.get(1..3).unwrap_or("0");
    let str_g: &str = hexcode.get(3..5).unwrap_or("0");
    let str_b: &str = hexcode.get(5..7).unwrap_or("0");

    let r: u8 = u8::from_str_radix(str_r, 16).unwrap_or(0);
    let g: u8 = u8::from_str_radix(str_g, 16).unwrap_or(0);
    let b: u8 = u8::from_str_radix(str_b, 16).unwrap_or(0);

    Color::RGB(r, g, b).normal()
}

pub fn apply_color<'a>(
    x: String,
    c: &ConfigColorByTheme,
    theme: &ConfigTheme,
    faded: bool,
) -> ANSIGenericString<'a, str> {
    let c = match theme {
        ConfigTheme::Dark => &c.dark,
        ConfigTheme::Light => &c.light,
        _ => unreachable!(),
    };

    if faded {
        match c {
            ConfigColor::BrightBlack => BLACK.paint(x),
            ConfigColor::BrightRed => RED.paint(x),
            ConfigColor::BrightGreen => GREEN.paint(x),
            ConfigColor::BrightYellow => YELLOW.paint(x),
            ConfigColor::BrightBlue => BLUE.paint(x),
            ConfigColor::BrightMagenta => MAGENTA.paint(x),
            ConfigColor::BrightCyan => CYAN.paint(x),
            ConfigColor::BrightWhite => WHITE.paint(x),
            ConfigColor::Black => BLACK.paint(x),
            ConfigColor::Red => RED.paint(x),
            ConfigColor::Green => GREEN.paint(x),
            ConfigColor::Yellow => YELLOW.paint(x),
            ConfigColor::Blue => BLUE.paint(x),
            ConfigColor::Magenta => MAGENTA.paint(x),
            ConfigColor::Cyan => CYAN.paint(x),
            ConfigColor::White => WHITE.paint(x),
            ConfigColor::Color256(c) => Fixed(*c).paint(x),
            ConfigColor::RGB(c) => hexcode2color(c).paint(x),
        }
    } else {
        match c {
            ConfigColor::BrightBlack => BRIGHT_BLACK.paint(x),
            ConfigColor::BrightRed => BRIGHT_RED.paint(x),
            ConfigColor::BrightGreen => BRIGHT_GREEN.paint(x),
            ConfigColor::BrightYellow => BRIGHT_YELLOW.paint(x),
            ConfigColor::BrightBlue => BRIGHT_BLUE.paint(x),
            ConfigColor::BrightMagenta => BRIGHT_MAGENTA.paint(x),
            ConfigColor::BrightCyan => BRIGHT_CYAN.paint(x),
            ConfigColor::BrightWhite => BRIGHT_WHITE.paint(x),
            ConfigColor::Black => BLACK.paint(x),
            ConfigColor::Red => RED.paint(x),
            ConfigColor::Green => GREEN.paint(x),
            ConfigColor::Yellow => YELLOW.paint(x),
            ConfigColor::Blue => BLUE.paint(x),
            ConfigColor::Magenta => MAGENTA.paint(x),
            ConfigColor::Cyan => CYAN.paint(x),
            ConfigColor::White => WHITE.paint(x),
            ConfigColor::Color256(c) => Fixed(*c).paint(x),
            ConfigColor::RGB(c) => hexcode2color(c).paint(x),
        }
    }
}

pub fn apply_style<'a>(
    x: String,
    cs: &ConfigColumnStyle,
    s: &ConfigStyle,
    theme: &ConfigTheme,
    faded: bool,
) -> ANSIGenericString<'a, str> {
    match cs {
        ConfigColumnStyle::Fixed(c) => apply_color(x, c, theme, faded),
        ConfigColumnStyle::ByPercentage => apply_style_by_percentage(x, s, theme, faded),
        ConfigColumnStyle::ByState => apply_style_by_state(x, s, theme, faded),
        ConfigColumnStyle::ByUnit => apply_style_by_unit(x, s, theme, faded),
    }
}

pub fn color_to_column_style(c: &ConfigColorByTheme) -> ConfigColumnStyle {
    ConfigColumnStyle::Fixed(c.clone())
}
