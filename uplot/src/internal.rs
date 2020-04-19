use crate::config::Config;
use core::cmp;
use embedded_graphics::{
    fonts::{Font, Font12x16, Text},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, Rectangle},
    style::{PrimitiveStyle, PrimitiveStyleBuilder, TextStyle, TextStyleBuilder},
};

#[derive(Debug)]
pub(crate) struct InternalConfig<'cfg> {
    pub cfg: Config<'cfg>,
    pub center: Point,
    pub x_from_range: (i32, i32),
    pub x_to_range: (i32, i32),
    pub y_from_range: (i32, i32),
    pub y_to_range: (i32, i32),
    pub bg_style: PrimitiveStyle<Rgb888>,
    pub background: Rectangle,
    pub grid_horiz_origin_line: Line,
    pub grid_line_style: PrimitiveStyle<Rgb888>,
    pub axis_text_style: TextStyle<Rgb888, Font12x16>,
    pub x_axis_text: Text<'cfg>,
    pub y_axis_text: Text<'cfg>,
}

impl<'cfg> InternalConfig<'cfg> {
    pub fn new(cfg: Config<'cfg>, x_max: i32) -> Self {
        let text_offset = cmp::max(1, i32::from(cfg.border_stroke) / 2);

        let center = cfg.top_left + (cfg.bottom_right - cfg.top_left) / 2;

        let x_from_range = (0, x_max);
        let x_to_range = (
            cfg.top_left.x + i32::from(cfg.border_stroke),
            cfg.bottom_right.x - i32::from(cfg.border_stroke),
        );

        let y_from_range = (cfg.y_min, cfg.y_max);
        let y_to_range = (
            cfg.bottom_right.y - i32::from(cfg.border_stroke),
            cfg.top_left.y + i32::from(cfg.border_stroke),
        );

        let bg_style = PrimitiveStyleBuilder::new()
            .stroke_color(cfg.border_stroke_color)
            .stroke_width(cfg.border_stroke.into())
            .fill_color(cfg.bg_color)
            .build();
        let background = Rectangle::new(cfg.top_left, cfg.bottom_right);

        let grid_line_style = PrimitiveStyle::with_stroke(cfg.grid_color, 1);

        let grid_horiz_origin_line = Line::new(
            Point::new(i32::from(cfg.border_stroke), center.y),
            Point::new(cfg.bottom_right.x - i32::from(cfg.border_stroke), center.y),
        );

        let font = Font12x16;
        let char_size = Font12x16::CHARACTER_SIZE;
        let char_height = char_size.height as i32;
        let axis_text_style = TextStyleBuilder::new(font)
            .text_color(cfg.axis_label_color)
            .background_color(cfg.axis_label_bg_color)
            .build();

        let x_axis_text = Text::new(
            &cfg.x_axis_lable,
            Point::new(
                center.x,
                cfg.bottom_right.y - char_height - i32::from(cfg.border_stroke) - text_offset,
            ),
        );

        let y_axis_text = Text::new(
            &cfg.y_axis_lable,
            Point::new(
                cfg.top_left.x + i32::from(cfg.border_stroke) + text_offset,
                center.y - (char_height / 2),
            ),
        );

        InternalConfig {
            cfg,
            center,
            x_from_range,
            x_to_range,
            y_from_range,
            y_to_range,
            bg_style,
            background,
            grid_line_style,
            grid_horiz_origin_line,
            axis_text_style,
            x_axis_text,
            y_axis_text,
        }
    }
}
