#![no_std]

pub use crate::storage::Storage;
use embedded_graphics::{
    fonts::{Font, Font12x16, Text},
    pixelcolor::{Rgb888, RgbColor},
    prelude::*,
    primitives::{Line, Rectangle},
    style::{PrimitiveStyle, PrimitiveStyleBuilder, TextStyle},
};
use generic_array::ArrayLength;

mod storage;

//
// - horiz/vert grid lines
// - scale and offset
// - colors / fills / stroke
// - style/etc points/lines
// - labeling
// - samples buffer
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Config<'cfg> {
    pub top_left: Point,
    pub bottom_right: Point,
    pub y_min: i32,
    pub y_max: i32,
    pub y_ticks: u16,
    pub border_stroke: u16,
    pub x_axis_lable: &'cfg str,
    pub y_axis_lable: &'cfg str,
}

impl<'cfg> Default for Config<'cfg> {
    fn default() -> Self {
        Config {
            top_left: Point::new(0, 0),
            bottom_right: Point::new(800, 480),
            y_min: core::i8::MIN.into(),
            y_max: core::i8::MAX.into(),
            y_ticks: 0,
            border_stroke: 2,
            x_axis_lable: "X",
            y_axis_lable: "Y",
        }
    }
}

pub struct Plot<'cfg, T, N>
where
    N: ArrayLength<T>,
    T: Copy + Into<i32>,
{
    config: Config<'cfg>,
    storage: Storage<T, N>,
}

impl<'cfg, T, N> Plot<'cfg, T, N>
where
    N: ArrayLength<T>,
    T: Copy + Into<i32>,
{
    pub fn new(config: Config<'cfg>, storage: Storage<T, N>) -> Self {
        Plot { config, storage }
    }
}

impl<'cfg, T, N> Plot<'cfg, T, N>
where
    N: ArrayLength<T>,
    T: Copy + Into<i32>,
{
    pub fn add_measurement(&mut self, t: T) {
        self.storage.write(t);
    }

    pub fn build(&'cfg self) -> impl Iterator<Item = Pixel<Rgb888>> + 'cfg {
        // TODO - precompute these in Config
        let center = (self.config.bottom_right - self.config.top_left) / 2;

        let x_max = self.storage.capacity() as i32;
        let x_from_range = (0, x_max);
        let x_to_range = (
            i32::from(self.config.border_stroke),
            self.config.bottom_right.x - i32::from(self.config.border_stroke),
        );

        let y_from_range = (self.config.y_min, self.config.y_max);
        let y_to_range = (
            self.config.bottom_right.y - i32::from(self.config.border_stroke),
            i32::from(self.config.border_stroke),
        );

        let bg_style = PrimitiveStyleBuilder::new()
            .stroke_color(Rgb888::BLACK)
            .stroke_width(self.config.border_stroke.into())
            .fill_color(Rgb888::BLACK)
            .build();
        let background =
            Rectangle::new(self.config.top_left, self.config.bottom_right).into_styled(bg_style);

        let grid_color = Rgb888::new(30, 30, 30);
        let grid_line_style = PrimitiveStyle::with_stroke(grid_color, 1);

        // TODO - config origin
        // y == 0
        let grid_horiz_origin_line = Line::new(
            Point::new(i32::from(self.config.border_stroke), center.y),
            Point::new(
                self.config.bottom_right.x - i32::from(self.config.border_stroke),
                center.y,
            ),
        )
        .into_styled(grid_line_style);

        // TODO - need some x axis units / type
        background
            .into_iter()
            .chain(&grid_horiz_origin_line)
            .chain(Self::labels(&self.config))
            .chain(self.storage.into_iter().enumerate().map(move |(idx, t)| {
                // TODO - proper scale/offset/transform function
                let x = scale(idx as i32, x_from_range, x_to_range);
                let y = scale(Into::<i32>::into(t), y_from_range, y_to_range);
                Pixel(Point::new(x, y), Rgb888::GREEN)
            }))
    }

    pub fn labels(cfg: &Config<'cfg>) -> impl Iterator<Item = Pixel<Rgb888>> + 'cfg {
        // TODO - precompute these in Config
        let center = (cfg.bottom_right - cfg.top_left) / 2;
        let font = Font12x16;
        let char_size = Font12x16::CHARACTER_SIZE;
        let char_height = char_size.height as i32;
        let text_style = TextStyle::new(font, RgbColor::WHITE);

        let x_axis = Text::new(
            &cfg.x_axis_lable,
            Point::new(
                center.x,
                cfg.bottom_right.y - char_height - i32::from(cfg.border_stroke),
            ),
        )
        .into_styled(text_style)
        .into_iter();
        let y_axis = Text::new(
            &cfg.y_axis_lable,
            Point::new(i32::from(cfg.border_stroke), center.y - (char_height / 2)),
        )
        .into_styled(text_style)
        .into_iter();
        x_axis.chain(y_axis)
    }
}

fn scale(x: i32, from_range: (i32, i32), to_range: (i32, i32)) -> i32 {
    let from = (from_range.0 as f32, from_range.1 as f32);
    let to = (to_range.0 as f32, to_range.1 as f32);
    let sx = map_range(x as f32, from, to);
    sx as i32
}

fn map_range(s: f32, from_range: (f32, f32), to_range: (f32, f32)) -> f32 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use generic_array::typenum::U12;

    #[test]
    fn basic_usage() {
        let storage = Storage::<i8, U12>::new();
        let config = Config {
            top_left: Point::new(0, 0),
            bottom_right: Point::new(800, 480),
            y_min: core::i8::MIN.into(),
            y_max: core::i8::MAX.into(),
            ..Default::default()
        };
        let mut plot = Plot::new(config, storage);
        for t in &[1, 2, 3, 4, 5] {
            plot.add_measurement(*t);
        }
    }
}
