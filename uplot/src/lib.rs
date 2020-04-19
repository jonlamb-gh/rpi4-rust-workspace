#![no_std]

// TODO
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
use embedded_graphics::{
    fonts::Text,
    pixelcolor::{Rgb888, RgbColor},
    prelude::*,
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
    }

    pub fn build(&'cfg self) -> impl Iterator<Item = Pixel<Rgb888>> + 'cfg {
        let background = self.config.background.into_styled(self.config.bg_style);
        background
            .into_iter()
            .chain(Self::grid(&self.config))
            .chain(Self::axis_labels(&self.config))
            .chain(Self::labels(&self.config))
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
        cfg.label_storage.iter().flat_map(move |label| {
            let x = label.x_to;
            let y = scale(label.y_from, cfg.y_from_range, cfg.y_to_range);
            let text = Text::new(&label.string, Point::new(x, y));
            text.into_styled(cfg.label_text_style).into_iter()
        })
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
