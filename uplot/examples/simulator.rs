// TODO
// refactor this to be a proper example
//
// cargo run --example simulator --target x86_64-unknown-linux-gnu

use embedded_graphics::{pixelcolor::Rgb888, prelude::*};
use embedded_graphics_simulator::{
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use generic_array::typenum::U256;
use rand::{thread_rng, Rng};
use std::thread;
use std::time::Duration;
use uplot::{Config, LabelStorage, Plot, Storage};

fn main() -> Result<(), std::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(1600, 960));

    let output_settings = OutputSettingsBuilder::new().build();

    let mut win = Window::new("Plot", &output_settings);

    let config = Config {
        top_left: Point::new(0, 0),
        bottom_right: Point::new(800, 480),
        y_min: std::i8::MIN.into(),
        y_max: std::i8::MAX.into(),
        ..Default::default()
    };
    let mut plot_tl = Plot::new(config, LabelStorage::new(), Storage::<i8, U256>::new());

    let config = Config {
        top_left: Point::new(800, 0),
        bottom_right: Point::new(1600, 480),
        y_min: std::i8::MIN.into(),
        y_max: std::i8::MAX.into(),
        ..Default::default()
    };
    let mut plot_tr = Plot::new(config, LabelStorage::new(), Storage::<i8, U256>::new());

    let config = Config {
        top_left: Point::new(0, 480),
        bottom_right: Point::new(800, 960),
        y_min: std::i8::MIN.into(),
        y_max: std::i8::MAX.into(),
        ..Default::default()
    };
    let mut plot_bl = Plot::new(config, LabelStorage::new(), Storage::<i8, U256>::new());

    let config = Config {
        top_left: Point::new(800, 480),
        bottom_right: Point::new(1600, 960),
        y_min: std::i8::MIN.into(),
        y_max: std::i8::MAX.into(),
        ..Default::default()
    };
    let mut plot_br = Plot::new(config, LabelStorage::new(), Storage::<i8, U256>::new());

    let mut rng = thread_rng();
    let mut m: i8 = 0;
    'running: loop {
        let delta: i8 = rng.gen_range(-3, 4);
        m = m.wrapping_add(delta);
        plot_tl.add_measurement(m);
        plot_tr.add_measurement(m);
        plot_bl.add_measurement(m);
        plot_br.add_measurement(m);

        plot_tl.build().draw(&mut display)?;
        plot_tr.build().draw(&mut display)?;
        plot_bl.build().draw(&mut display)?;
        plot_br.build().draw(&mut display)?;

        win.update(&display);

        for event in win.events() {
            match event {
                SimulatorEvent::MouseButtonUp { point, .. } => {
                    println!("Click event at ({}, {})", point.x, point.y);
                }
                SimulatorEvent::Quit => break 'running,
                _ => {}
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    Ok(())
}
