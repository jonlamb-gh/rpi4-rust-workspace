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
use uplot::{Config, Plot, Storage};

//use embedded_graphics::{
//    fonts::{Font6x8, Text},
//    pixelcolor::{Rgb888, RgbColor},
//    prelude::*,
//    primitives::{Circle, Rectangle, Triangle},
//    style::{PrimitiveStyle, TextStyle},
//};

fn main() -> Result<(), std::convert::Infallible> {
    let mut display: SimulatorDisplay<Rgb888> = SimulatorDisplay::new(Size::new(800, 480));

    // Create styles used by the drawing operations.
    //    let thin_stroke = PrimitiveStyle::with_stroke(RgbColor::BLUE, 1);
    //    let thick_stroke = PrimitiveStyle::with_stroke(RgbColor::BLUE, 3);
    //    let fill = PrimitiveStyle::with_fill(RgbColor::BLUE);
    //    let text_style = TextStyle::new(Font6x8, RgbColor::BLUE);

    //let yoffset = 10;

    // Draw a 3px wide outline around the display.
    //Rectangle::new(
    //    Point::zero(),
    //    Point::new(display.size().width as _, display.size().height as _),
    //)
    //.into_styled(thick_stroke)
    //.draw(&mut display)?;

    // Draw a triangle.
    //Triangle::new(
    //    Point::new(16, 16 + yoffset),
    //    Point::new(16 + 16, 16 + yoffset),
    //    Point::new(16 + 8, yoffset),
    //)
    //.into_styled(thin_stroke)
    //.draw(&mut display)?;

    // Draw a filled square
    //    Rectangle::new(Point::new(52, yoffset), Point::new(16, 16))
    //        .into_styled(fill)
    //        .draw(&mut display)?;

    // Draw a circle with a 3px wide stroke.
    //Circle::new(Point::new(88, yoffset), 17)
    //    .into_styled(thick_stroke)
    //    .draw(&mut display)?;

    // Draw centered text.
    //let text = "embedded-graphics";
    //let width = text.len() as i32 * 6;
    //Text::new(text, Point::new(64 - width / 2, 40))
    //    .into_styled(text_style)
    //    .draw(&mut display)?;

    let output_settings = OutputSettingsBuilder::new()
        //.theme(RgbColorTheme::BLUE)
        .build();

    let mut win = Window::new("Plot", &output_settings);

    let storage = Storage::<i8, U256>::new();

    let config = Config {
        top_left: Point::new(0, 0),
        bottom_right: Point::new(800, 480),
        y_min: std::i8::MIN.into(),
        y_max: std::i8::MAX.into(),
        ..Default::default()
    };

    let mut plot = Plot::new(config, storage);

    let mut rng = thread_rng();

    let mut m: i8 = 0;

    'running: loop {
        let delta: i8 = rng.gen_range(-3, 4);
        m = m.wrapping_add(delta);
        plot.add_measurement(m);

        plot.build().draw(&mut display)?;

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
