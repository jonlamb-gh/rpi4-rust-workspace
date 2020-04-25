use crate::{frame, Config, Label, LabelStorage, Point2D, Position, Range1D};
use core::cmp;
use core::fmt::Write;
use embedded_graphics::{
    fonts::{Font, Font6x6, Font8x16, Text},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{Line, Rectangle},
    style::{PrimitiveStyle, PrimitiveStyleBuilder, TextStyle, TextStyleBuilder},
};

#[derive(Debug)]
pub(crate) struct InternalConfig<'cfg> {
    pub cfg: Config<'cfg>,
    pub center: Point2D<frame::Window>,
    pub x_from: Range1D<frame::World>,
    pub x_to: Range1D<frame::Window>,
    pub y_from: Range1D<frame::World>,
    pub y_to: Range1D<frame::Window>,
    pub bg_style: PrimitiveStyle<Rgb888>,
    pub background: Rectangle,
    pub grid_line_style: PrimitiveStyle<Rgb888>,
    pub grid_horiz_origin_line: Line,
    pub grid_horiz_out_line: Line,
    pub grid_vert_out_line: Line,
    pub axis_text_style: TextStyle<Rgb888, Font8x16>,
    pub x_axis_text: Text<'cfg>,
    pub y_axis_text: Text<'cfg>,
    pub label_text_style: TextStyle<Rgb888, Font6x6>,
    pub label_text_pos: Point2D<frame::Window>,
    pub label_tick_x: Position<frame::Window>,
    pub label_storage: LabelStorage,
}

impl<'cfg> InternalConfig<'cfg> {
    pub fn new(cfg: Config<'cfg>, label_storage: LabelStorage, x_in_max: i32) -> Self {
        // TODO - move to config
        let font = Font8x16;
        let char_size = Font8x16::CHARACTER_SIZE;
        let char_width = char_size.width as i32;
        let char_height = char_size.height as i32;

        let text_offset = cmp::max(1, i32::from(cfg.border_stroke) / 2);

        let x_min = cfg.top_left.x + i32::from(cfg.border_stroke) + char_width + text_offset;
        let x_max = cfg.bottom_right.x - i32::from(cfg.border_stroke);

        let y_min = cfg.top_left.y + i32::from(cfg.border_stroke);
        let y_max = cfg.bottom_right.y - char_height - i32::from(cfg.border_stroke) - text_offset;

        let center = Point2D::new(cfg.top_left + (cfg.bottom_right - cfg.top_left) / 2);

        // TODO - move to config
        let label_font = Font6x6;
        let label_tick_x = Position::new(x_min + text_offset);
        let label_text_style = TextStyleBuilder::new(label_font)
            .text_color(cfg.label_color)
            .background_color(cfg.label_bg_color)
            .build();
        let label_text_pos = Point::new(
            x_min + cfg.label_line_len + i32::from(cfg.border_stroke),
            -(Font6x6::CHARACTER_SIZE.height as i32),
        )
        .into();

        let x_from = (0, x_in_max).into();
        let x_to = (x_min, x_max).into();

        let y_from = (cfg.y_min, cfg.y_max).into();
        let y_to = (y_max, y_min).into();

        let bg_style = PrimitiveStyleBuilder::new()
            .stroke_color(cfg.border_stroke_color)
            .stroke_width(cfg.border_stroke.into())
            .fill_color(cfg.bg_color)
            .build();
        let background = Rectangle::new(cfg.top_left, cfg.bottom_right);

        let grid_line_style = PrimitiveStyle::with_stroke(cfg.grid_color, 1);

        let grid_horiz_origin_line =
            Line::new(Point::new(x_min, center.p.y), Point::new(x_max, center.p.y));

        let grid_horiz_out_line = Line::new(Point::new(x_min, y_max), Point::new(x_max, y_max));

        let grid_vert_out_line = Line::new(Point::new(x_min, y_min), Point::new(x_min, y_max));

        let axis_text_style = TextStyleBuilder::new(font)
            .text_color(cfg.axis_label_color)
            .background_color(cfg.axis_label_bg_color)
            .build();

        let x_axis_text = Text::new(
            &cfg.x_axis_lable,
            Point::new(
                center.p.x,
                cfg.bottom_right.y - char_height - i32::from(cfg.border_stroke) - text_offset,
            ),
        );

        let y_axis_text = Text::new(
            &cfg.y_axis_lable,
            Point::new(
                cfg.top_left.x + i32::from(cfg.border_stroke) + text_offset,
                center.p.y - (char_height / 2),
            ),
        );

        let mut icfg = InternalConfig {
            cfg,
            center,
            x_from,
            x_to,
            y_from,
            y_to,
            bg_style,
            background,
            grid_line_style,
            grid_horiz_origin_line,
            grid_horiz_out_line,
            grid_vert_out_line,
            axis_text_style,
            x_axis_text,
            y_axis_text,
            label_text_style,
            label_text_pos,
            label_tick_x,
            label_storage,
        };

        if icfg.cfg.label_y_ticks != 0 {
            icfg.generate_labels();
        }

        icfg
    }

    // TODO
    // - error handling
    pub fn generate_labels(&mut self) {
        self.label_storage.labels.clear();

        let y_from_offset = i32::from(self.cfg.label_y_ticks);

        let y_from_origin = if self.y_from.r.0 < 0 {
            // TODO
            self.y_from.r.0 + self.y_from.r.1 + 1
        } else {
            self.y_from.r.0
        };

        // Skip origin marker
        let y_from_start = y_from_origin + y_from_offset;

        for y_from in (y_from_start..self.cfg.y_max).step_by(usize::from(self.cfg.label_y_ticks)) {
            let y_pos: Position<frame::World> = y_from.into();
            let x_win_pos: Position<frame::Window> = self.label_tick_x.into();
            let y_win_pos: Position<frame::Window> = self.y_from.scale(y_pos, &self.y_to);
            let mut label = Label {
                pos: (x_win_pos, y_win_pos).into(),
                ..Default::default()
            };

            write!(&mut label.string, "{}", y_pos.p).ok();
            self.label_storage.labels.push(label).ok();

            if y_from_origin != self.y_from.r.0 {
                let y_pos: Position<frame::World> = (-y_from).into();
                let y_win_pos: Position<frame::Window> = self.y_from.scale(y_pos, &self.y_to);
                let mut label = Label {
                    pos: (x_win_pos, y_win_pos).into(),
                    ..Default::default()
                };
                write!(&mut label.string, "{}", y_pos.p).ok();
                self.label_storage.labels.push(label).ok();
            }
        }
    }
}
