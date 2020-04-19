use embedded_graphics::{
    pixelcolor::{Rgb888, RgbColor},
    prelude::*,
};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Config<'cfg> {
    pub top_left: Point,
    pub bottom_right: Point,
    pub y_min: i32,
    pub y_max: i32,
    pub x_axis_lable: &'cfg str,
    pub y_axis_lable: &'cfg str,
    pub border_stroke: u16,
    pub border_stroke_color: Rgb888,
    pub bg_color: Rgb888,
    pub grid_color: Rgb888,
    pub axis_label_color: Rgb888,
    pub axis_label_bg_color: Rgb888,
    pub label_color: Rgb888,
    pub label_bg_color: Rgb888,
    pub label_line_color: Option<Rgb888>,
    pub label_line_len: i32,
    /// If zero, no Y axis labels, same input units as y_min/y_max
    pub label_y_ticks: u16,
}

impl<'cfg> Default for Config<'cfg> {
    fn default() -> Self {
        Config {
            top_left: Point::new(0, 0),
            bottom_right: Point::new(800, 480),
            y_min: core::i8::MIN.into(),
            y_max: core::i8::MAX.into(),
            x_axis_lable: "X",
            y_axis_lable: "Y",
            border_stroke: 2,
            border_stroke_color: Rgb888::BLACK,
            bg_color: Rgb888::BLACK,
            grid_color: Rgb888::new(30, 30, 30),
            axis_label_color: Rgb888::WHITE,
            axis_label_bg_color: Rgb888::BLACK,
            label_color: Rgb888::WHITE,
            label_bg_color: Rgb888::BLACK,
            label_line_color: Some(Rgb888::RED),
            label_line_len: 32,
            label_y_ticks: 20,
        }
    }
}
