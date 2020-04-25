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
pub use crate::coordinates::{frame, Point2D, Position, Range1D};
use crate::internal::InternalConfig;
pub use crate::label_storage::{Label, LabelStorage};
pub use crate::storage::Storage;
pub use crate::string::{PlotString, PlotStringCapacity};
use core::fmt::Write;
use embedded_graphics::{
    fonts::Text, pixelcolor::Rgb888, prelude::*, primitives::Line, style::PrimitiveStyle,
};
use generic_array::ArrayLength;

mod config;
mod coordinates;
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
                let x = self
                    .config
                    .x_from
                    .scale((idx as i32).into(), &self.config.x_to);
                let value = Into::<i32>::into(t);
                let y = self.config.y_from.scale(value.into(), &self.config.y_to);
                Pixel(Point::new(x.p, y.p), self.config.cfg.points_color)
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
            let text = Text::new(&label.string, label.pos.into());
            text.into_styled(cfg.label_text_style).into_iter()
        })
    }

    fn label_line(
        cfg: &InternalConfig<'cfg>,
        recent: i32,
    ) -> Option<impl Iterator<Item = Pixel<Rgb888>>> {
        if let Some(c) = &cfg.cfg.label_line_color {
            let x0 = cfg.x_to.r.0;
            let x1 = x0 + cfg.cfg.label_line_len;
            let y = cfg.y_from.scale(recent.into(), &cfg.y_to);
            Some(
                Line::new(Point::new(x0, y.p), Point::new(x1, y.p))
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
        if cfg.cfg.label_line_color.is_some() {
            let x: Position<frame::Window> = cfg.label_text_pos.p.x.into();
            let y = cfg.y_from.scale(recent.into(), &cfg.y_to);
            let text = Text::new(&string, Point::new(x.p, y.p));
            Some(text.into_styled(cfg.label_text_style).into_iter())
        } else {
            None
        }
    }
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
