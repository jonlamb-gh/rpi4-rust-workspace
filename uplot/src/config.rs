use crate::{frame, Point2D, Position};
use embedded_graphics::pixelcolor::{Rgb888, RgbColor};

// TODO - FontConfig

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum PlotStyle {
    Points,
}

impl Default for PlotStyle {
    fn default() -> Self {
        PlotStyle::Points
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Config<'cfg> {
    pub layout: LayoutConfig,
    pub style: StyleConfig,
    pub label: LabelConfig<'cfg>,
}

impl<'cfg> Default for Config<'cfg> {
    fn default() -> Self {
        Config {
            layout: LayoutConfig::default(),
            style: StyleConfig::default(),
            label: LabelConfig::default(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LayoutConfig {
    pub top_left: Point2D<frame::Window>,
    pub bottom_right: Point2D<frame::Window>,
    pub y_min: Position<frame::World>,
    pub y_max: Position<frame::World>,
    pub border_stroke: u16,
    pub border_stroke_color: Rgb888,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        LayoutConfig {
            top_left: (0_i32, 0_i32).into(),
            bottom_right: (800_i32, 480_i32).into(),
            y_min: i32::from(core::i8::MIN).into(),
            y_max: i32::from(core::i8::MAX).into(),
            border_stroke: 2,
            border_stroke_color: Rgb888::BLACK,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct StyleConfig {
    pub style: PlotStyle,
    pub points_color: Rgb888,
    pub border_stroke: u16,
    pub border_stroke_color: Rgb888,
    pub bg_color: Rgb888,
    pub grid_color: Rgb888,
    pub axis_label_color: Rgb888,
    pub axis_label_bg_color: Rgb888,
}

impl Default for StyleConfig {
    fn default() -> Self {
        StyleConfig {
            style: PlotStyle::default(),
            points_color: Rgb888::GREEN,
            border_stroke: 2,
            border_stroke_color: Rgb888::BLACK,
            bg_color: Rgb888::BLACK,
            grid_color: Rgb888::new(30, 30, 30),
            axis_label_color: Rgb888::WHITE,
            axis_label_bg_color: Rgb888::BLACK,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct LabelConfig<'cfg> {
    pub x_axis_label: &'cfg str,
    pub y_axis_label: &'cfg str,
    pub label_line_len: i32,
    /// If zero, no Y axis labels, same input units as y_min/y_max
    pub y_ticks: u16,
    pub color: Rgb888,
    pub bg_color: Rgb888,
    pub line_color: Option<Rgb888>,
}

impl<'cfg> Default for LabelConfig<'cfg> {
    fn default() -> Self {
        LabelConfig {
            x_axis_label: "X",
            y_axis_label: "Y",
            label_line_len: 32,
            y_ticks: 20,
            color: Rgb888::WHITE,
            bg_color: Rgb888::BLACK,
            line_color: Some(Rgb888::RED),
        }
    }
}
