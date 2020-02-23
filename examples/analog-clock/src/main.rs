//! A port of the analog-clock embedded-graphics example
//! https://github.com/jamwaffles/embedded-graphics/blob/master/simulator/examples/analog-clock.rs

#![no_std]
#![no_main]

extern crate bcm2711_hal as hal;

use crate::hal::bcm2711::dma::{Enable, DMA};
use crate::hal::bcm2711::gpio::GPIO;
use crate::hal::bcm2711::mbox::MBOX;
use crate::hal::bcm2711::uart1::UART1;
use crate::hal::clocks::Clocks;
use crate::hal::dma;
use crate::hal::mailbox::*;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::time::Bps;
use core::f32::consts::{FRAC_PI_2, PI};
use core::fmt::Write;
use display::embedded_graphics::prelude::*;
use display::embedded_graphics::{
    egcircle,
    fonts::{Font12x16, Text},
    pixelcolor::Rgb888,
    primitive_style,
    primitives::{Circle, Line, Rectangle},
    style::{PrimitiveStyle, PrimitiveStyleBuilder, Styled, TextStyle},
};
use display::{Display, SCRATCHPAD_MEM_MIN_SIZE};
use heapless::consts::U256 as StringCapacity;
use heapless::String;
use micromath::F32Ext;

/// The width and height of the simulated display
const DISPLAY_WIDTH: i32 = 800;
const DISPLAY_HEIGHT: i32 = 480;

/// The center of the clock face
const CENTER: Point = Point::new(DISPLAY_WIDTH / 2, DISPLAY_HEIGHT / 2);

/// The radius of the clock face
const SIZE: u32 = 200;

/// Start at the top of the circle
const START: f32 = -FRAC_PI_2;

fn kernel_entry() -> ! {
    let mut mbox = Mailbox::new(MBOX::new());
    let clocks = Clocks::freeze(&mut mbox).unwrap();
    let gpio = GPIO::new();
    let gp = gpio.split();

    let tx = gp.p14.into_alternate_af5();
    let rx = gp.p15.into_alternate_af5();

    let mut serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200), clocks);

    writeln!(serial, "Embedded graphics example").ok();

    // Construct the DMA peripheral, reset and enable CH0
    let dma = DMA::new();
    let mut dma_parts = dma.split();
    dma_parts.enable.enable.modify(Enable::En0::Set);
    let mut dma_chan = dma_parts.ch0;
    dma_chan.reset();

    writeln!(serial, "DMA Channel ID: 0x{:X}", dma_chan.id()).ok();

    let sn = get_serial_number(&mut mbox).serial_number();
    writeln!(serial, "Serial number: {:#010X}", sn).ok();

    writeln!(serial, "Requesting default framebuffer allocation").ok();

    let fb = alloc_framebuffer(&mut mbox);

    writeln!(
        serial,
        "  width: {} height: {} pitch {} {:?}",
        fb.virt_width,
        fb.virt_height,
        fb.pitch(),
        fb.pixel_order,
    )
    .ok();

    assert_eq!(fb.virt_width, DISPLAY_WIDTH as usize);
    assert_eq!(fb.virt_height, DISPLAY_HEIGHT as usize);

    writeln!(
        serial,
        "  address: {:#010X} bus_address: {:#010X} size: 0x{:X}",
        fb.alloc_buffer_address(),
        fb.alloc_buffer_bus_address(),
        fb.alloc_buffer_size()
    )
    .ok();

    let vc_mem_size = fb.alloc_buffer_size();
    let vc_mem_words = vc_mem_size / 4;
    writeln!(serial, "  bytes {} - words {}", vc_mem_size, vc_mem_words,).ok();
    let frontbuffer_mem = unsafe {
        core::slice::from_raw_parts_mut(fb.alloc_buffer_address() as *mut u32, vc_mem_words)
    };

    const STATIC_SIZE: usize = DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize * 4;
    assert!(vc_mem_size <= STATIC_SIZE);

    let dcb_mem = unsafe {
        static mut DCB_MEM: [dma::ControlBlock; 1] = [dma::ControlBlock::new()];
        &mut DCB_MEM[..]
    };

    let backbuffer_mem = unsafe {
        static mut BACKBUFFER_MEM: [u32; STATIC_SIZE / 4] = [0; STATIC_SIZE / 4];
        &mut BACKBUFFER_MEM[..]
    };

    let scratchpad_mem = unsafe {
        static mut SCRATCHPAD_MEM: [u32; SCRATCHPAD_MEM_MIN_SIZE / 4] =
            [0; SCRATCHPAD_MEM_MIN_SIZE / 4];
        &mut SCRATCHPAD_MEM[..]
    };

    let mut display = Display::new(
        fb,
        dma_chan,
        dcb_mem,
        scratchpad_mem,
        &mut backbuffer_mem[..vc_mem_words],
        &mut frontbuffer_mem[..vc_mem_words],
    )
    .unwrap();

    let mut strbuf: String<StringCapacity> = String::new();

    let mut mock_clock = MockClock::new();

    // Clear back and front buffers
    display.clear_screen().unwrap();

    loop {
        mock_clock.inc();

        strbuf.clear();
        write!(
            &mut strbuf,
            "{:02}:{:02}:{:02}",
            mock_clock.hour, mock_clock.minute, mock_clock.second
        )
        .unwrap();
        let digital_clock_text = strbuf.as_str();

        // Clear the backbuffer
        display.clear_buffer().unwrap();

        draw_face().draw(&mut display).unwrap();
        draw_hour_hand(mock_clock.hour).draw(&mut display).unwrap();
        draw_minute_hand(mock_clock.minute)
            .draw(&mut display)
            .unwrap();
        draw_seconds_hand(mock_clock.second)
            .draw(&mut display)
            .unwrap();

        // Draw digital clock just above center
        draw_digital_clock(&digital_clock_text)
            .draw(&mut display)
            .unwrap();

        // Draw a small circle over the hands in the center of the clock face. This has
        // to happen after the hands are drawn so they're covered up
        Circle::new(CENTER, 4)
            .into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE))
            .draw(&mut display)
            .unwrap();

        // DMA the backbuffer to the framebuffer
        display.swap_buffers().unwrap();
    }
}

struct MockClock {
    hour: u32,
    minute: u32,
    second: u32,
}

impl MockClock {
    pub fn new() -> Self {
        MockClock {
            hour: 0,
            minute: 0,
            second: 0,
        }
    }

    pub fn inc(&mut self) {
        self.second += 1;
        if self.second >= 60 {
            self.second = 0;
            self.minute += 1;
            if self.minute >= 60 {
                self.minute = 0;
                self.hour += 1;
                if self.hour >= 12 {
                    self.hour = 0;
                }
            }
        }
    }
}

fn get_serial_number(mbox: &mut Mailbox) -> GetSerialNumRepr {
    let resp = mbox
        .call(Channel::Prop, &GetSerialNumRepr::default())
        .expect("MBox call()");

    if let RespMsg::GetSerialNum(repr) = resp {
        repr
    } else {
        panic!("Invalid response\n{:#?}", resp);
    }
}

fn alloc_framebuffer(mbox: &mut Mailbox) -> AllocFramebufferRepr {
    let resp = mbox
        .call(Channel::Prop, &AllocFramebufferRepr::default())
        .expect("MBox call()");

    if let RespMsg::AllocFramebuffer(repr) = resp {
        repr
    } else {
        panic!("Invalid response\n{:#?}", resp);
    }
}

/// Convert a polar coordinate (angle/distance) into an (X, Y) coordinate
/// centered around `CENTER`
fn polar(angle: f32, radius: f32) -> Point {
    CENTER + Point::new((angle.cos() * radius) as i32, (angle.sin() * radius) as i32)
}

/// Draw a circle and 12 tics as a simple clock face
fn draw_face() -> impl Iterator<Item = Pixel<Rgb888>> {
    let tic_len = 10.0;

    // Use the circle macro to create the outer face
    let face = egcircle!(
        center = CENTER,
        radius = SIZE,
        style = primitive_style!(stroke_color = Rgb888::WHITE, stroke_width = 2)
    );

    // Create 12 `Line`s starting from the outer edge and drawing inwards by
    // `tic_len` pixels
    let tics = (0..12).into_iter().map(move |index| {
        // Start angle around the circle, in radians
        let angle = START + (PI * 2.0 / 12.0) * index as f32;

        // Start point on circumference
        let start = polar(angle, SIZE as f32);

        // End point; start point offset by `tic_len` pixels towards the circle center
        let end = polar(angle, SIZE as f32 - tic_len);

        Line::new(start, end)
            .into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
            .into_iter()
    });

    // Create a single iterator of pixels, first iterating over the circle, then
    // over the 12 lines generated
    face.into_iter().chain(tics.flatten())
}

/// Draw the seconds hand given a seconds value (0 - 59)
fn draw_seconds_hand(seconds: u32) -> impl Iterator<Item = Pixel<Rgb888>> {
    // Convert seconds into a position around the circle in radians
    let seconds_radians = ((seconds as f32 / 60.0) * 2.0 * PI) + START;

    let end = polar(seconds_radians, SIZE as f32);

    // Basic line hand
    let hand = Line::new(CENTER, end).into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1));

    // Decoration position
    let decoration_position = polar(seconds_radians, SIZE as f32 - 20.0);

    // Decoration style
    let decoration_style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb888::BLACK)
        .stroke_color(Rgb888::WHITE)
        .stroke_width(1)
        .build();

    // Add a fancy circle near the end of the hand
    let decoration = Circle::new(decoration_position, 5).into_styled(decoration_style);

    hand.into_iter().chain(&decoration)
}

/// Draw the hour hand (0-11)
fn draw_hour_hand(hour: u32) -> Styled<Line, PrimitiveStyle<Rgb888>> {
    // Convert hour into a position around the circle in radians
    let hour_radians = ((hour as f32 / 12.0) * 2.0 * PI) + START;

    let hand_len = SIZE as f32 - 60.0;

    let end = polar(hour_radians, hand_len);

    // Basic line hand
    Line::new(CENTER, end).into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
}

/// Draw the minute hand (0-59)
fn draw_minute_hand(minute: u32) -> Styled<Line, PrimitiveStyle<Rgb888>> {
    // Convert minute into a position around the circle in radians
    let minute_radians = ((minute as f32 / 60.0) * 2.0 * PI) + START;

    let hand_len = SIZE as f32 - 30.0;

    let end = polar(minute_radians, hand_len);

    // Basic line hand
    Line::new(CENTER, end).into_styled(PrimitiveStyle::with_stroke(Rgb888::WHITE, 1))
}

/// Draw digital clock just above center with black text on a white background
///
/// NOTE: The formatted time str must be passed in as references to temporary
/// values in a function can't be returned.
fn draw_digital_clock<'a>(time_str: &'a str) -> impl Iterator<Item = Pixel<Rgb888>> + 'a {
    let text = Text::new(&time_str, CENTER - Size::new(48, 48))
        .into_styled(TextStyle::new(Font12x16, Rgb888::BLACK));

    // Add a background around the time digits. Note that there is no
    // bottom-right padding as this is added by the font renderer itself
    let background = Rectangle::new(text.top_left() - Size::new(3, 3), text.bottom_right())
        .into_styled(PrimitiveStyle::with_fill(Rgb888::WHITE));

    // Draw the white background first, then the black text. Order matters here
    background.into_iter().chain(&text)
}

raspi3_boot::entry!(kernel_entry);
