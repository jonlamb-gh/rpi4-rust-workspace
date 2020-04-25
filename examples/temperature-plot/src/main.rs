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
use core::fmt::Write;
use display::embedded_graphics::prelude::*;
use display::{Display, SCRATCHPAD_MEM_MIN_SIZE};
use generic_array::typenum::U256;
use uplot::{Config, LabelStorage, LayoutConfig, Plot, Storage};

/// The width and height of the simulated display
const DISPLAY_WIDTH: i32 = 800;
const DISPLAY_HEIGHT: i32 = 480;

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

    let config = Config {
        layout: LayoutConfig {
            top_left: Point::new(0, 0).into(),
            bottom_right: Point::new(DISPLAY_WIDTH, DISPLAY_HEIGHT).into(),
            y_min: i32::from(0).into(),
            y_max: i32::from(101).into(),
            ..Default::default()
        },
        ..Default::default()
    };
    let mut plot = Plot::new(config, LabelStorage::new(), Storage::<u16, U256>::new());

    // Clear back and front buffers
    display.clear_screen().unwrap();

    loop {
        // Temperature of the SoC in thousandths of a degree C
        let temp_raw = get_temp(&mut mbox).temp();
        let temp = temp_raw / 1000;
        writeln!(serial, "temp {}", temp).ok();

        plot.add_measurement(temp as u16);

        // Clear the backbuffer
        display.clear_buffer().unwrap();

        // Render to the backbuffer
        plot.build().draw(&mut display).unwrap();

        // DMA the backbuffer to the framebuffer
        display.swap_buffers().unwrap();
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

fn get_temp(mbox: &mut Mailbox) -> GetTempRepr {
    let resp = mbox
        .call(Channel::Prop, &GetTempRepr::default())
        .expect("MBox call()");

    if let RespMsg::GetTemp(repr) = resp {
        repr
    } else {
        panic!("Invalid response\n{:#?}", resp);
    }
}

raspi3_boot::entry!(kernel_entry);
