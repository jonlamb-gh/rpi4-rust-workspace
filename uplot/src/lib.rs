#![no_std]

// TODO
// - rename to/from stuff to window/world coordinates
// - proper scale/offset/transform function
// - x axis units/data
// - horiz/vert grid lines
// - config origin, y == 0 or...
// - scale and offset
// - colors, fills, stroke
// - style/etc points/lines
// - labeling, fonts, text

pub use crate::config::Config;
use crate::internal::InternalConfig;
pub use crate::label_storage::LabelStorage;
pub use crate::storage::Storage;
pub use crate::string::{PlotString, PlotStringCapacity};
use core::fmt::Write;
use embedded_graphics::{
    fonts::Text,
    pixelcolor::{Rgb888, RgbColor},
    prelude::*,
    primitives::Line,
    style::PrimitiveStyle,
};
use generic_array::ArrayLength;

mod config;
mod internal;
mod label_storage;
mod storage;
mod string;

pub struct Plot<'cfg, T, N>
where
    N: ArrayLength<T>,
    T: Copy + Into<i32>,
{
    config: InternalConfig<'cfg>,
    storage: Storage<T, N>,
    recent: i32,
}

impl<'cfg, T, N> Plot<'cfg, T, N>
where
    N: ArrayLength<T>,
    T: Copy + Into<i32>,
{
    pub fn new(config: Config<'cfg>, label_storage: LabelStorage, storage: Storage<T, N>) -> Self {
        Plot {
            config: InternalConfig::new(config, label_storage, storage.capacity() as i32),
            storage,
            recent: 0,
        }
    }
}

impl<'cfg, T, N> Plot<'cfg, T, N>
where
    N: ArrayLength<T>,
    T: Copy + Into<i32>,
{
    pub fn add_measurement(&mut self, t: T) {
        self.storage.write(t);
        self.recent = Into::<i32>::into(*self.storage.recent());
        self.config.label_storage.value_label.clear();
        write!(
            &mut self.config.label_storage.value_label,
            "{}",
            self.recent
        )
        .ok();
    }

    pub fn build(&'cfg self) -> impl Iterator<Item = Pixel<Rgb888>> + 'cfg {
        let label_line_iter = Self::label_line(&self.config, self.recent);
        let label_text_iter = Self::label_text(
            &self.config,
            self.recent,
            &self.config.label_storage.value_label,
        );
        let background = self.config.background.into_styled(self.config.bg_style);
        background
            .into_iter()
            .chain(Self::grid(&self.config))
            .chain(Self::axis_labels(&self.config))
            .chain(Self::labels(&self.config))
            .chain(label_line_iter.into_iter().flat_map(|iter| iter))
            .chain(label_text_iter.into_iter().flat_map(|iter| iter))
            .chain(self.storage.into_iter().enumerate().map(move |(idx, t)| {
                let x = scale(idx as i32, self.config.x_from_range, self.config.x_to_range);
                let y = scale(
                    Into::<i32>::into(t),
                    self.config.y_from_range,
                    self.config.y_to_range,
                );
                Pixel(Point::new(x, y), Rgb888::GREEN)
            }))
    }

    fn grid(cfg: &InternalConfig<'cfg>) -> impl Iterator<Item = Pixel<Rgb888>> {
        // TODO - only need the origin line if T is signed, uses the horiz outline
        let grid_horiz_origin_line = cfg
            .grid_horiz_origin_line
            .into_styled(cfg.grid_line_style)
            .into_iter();
        let grid_horiz_out_line = cfg
            .grid_horiz_out_line
            .into_styled(cfg.grid_line_style)
            .into_iter();
        let grid_vert_out_line = cfg
            .grid_vert_out_line
            .into_styled(cfg.grid_line_style)
            .into_iter();
        grid_horiz_origin_line
            .chain(grid_horiz_out_line)
            .chain(grid_vert_out_line)
    }

    fn axis_labels(cfg: &InternalConfig<'cfg>) -> impl Iterator<Item = Pixel<Rgb888>> + 'cfg {
        let x_axis = cfg.x_axis_text.into_styled(cfg.axis_text_style).into_iter();
        let y_axis = cfg.y_axis_text.into_styled(cfg.axis_text_style).into_iter();
        x_axis.chain(y_axis)
    }

    fn labels(cfg: &'cfg InternalConfig<'cfg>) -> impl Iterator<Item = Pixel<Rgb888>> + 'cfg {
        cfg.label_storage.labels.iter().flat_map(move |label| {
            let x = label.x_to;
            let y = scale(label.y_from, cfg.y_from_range, cfg.y_to_range);
            let text = Text::new(&label.string, Point::new(x, y));
            text.into_styled(cfg.label_text_style).into_iter()
        })
    }

    fn label_line(
        cfg: &InternalConfig<'cfg>,
        recent: i32,
    ) -> Option<impl Iterator<Item = Pixel<Rgb888>>> {
        if let Some(c) = &cfg.cfg.label_line_color {
            let x0 = cfg.x_to_range.0;
            let x1 = x0 + cfg.cfg.label_line_len;
            let y = scale(recent, cfg.y_from_range, cfg.y_to_range);
            Some(
                Line::new(Point::new(x0, y), Point::new(x1, y))
                    .into_styled(PrimitiveStyle::with_stroke(*c, 1))
                    .into_iter(),
            )
        } else {
            None
        }
    }

    fn label_text<'a>(
        cfg: &InternalConfig<'cfg>,
        recent: i32,
        string: &'a str,
    ) -> Option<impl Iterator<Item = Pixel<Rgb888>> + 'a> {
        if let Some(_c) = &cfg.cfg.label_line_color {
            let x = cfg.x_to_range.0 + cfg.cfg.label_line_len + i32::from(cfg.cfg.border_stroke);
            let y = scale(recent, cfg.y_from_range, cfg.y_to_range);
            let text = Text::new(&string, Point::new(x, y));
            Some(text.into_styled(cfg.label_text_style).into_iter())
        } else {
            None
        }
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
        let config = Config {
            top_left: Point::new(0, 0),
            bottom_right: Point::new(800, 480),
            y_min: core::i8::MIN.into(),
            y_max: core::i8::MAX.into(),
            ..Default::default()
        };
        let mut plot = Plot::new(config, LabelStorage::new(), Storage::<i8, U12>::new());
        for t in &[1, 2, 3, 4, 5] {
            plot.add_measurement(*t);
        }
    }
}
