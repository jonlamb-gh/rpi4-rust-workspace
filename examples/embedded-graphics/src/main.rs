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
use display::embedded_graphics::{
    fonts::Font12x16, fonts::Text, pixelcolor::Rgb888, style::TextStyle,
};
use display::{Display, SCRATCHPAD_MEM_MIN_SIZE};

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

    const STATIC_SIZE: usize = 800 * 600 * 4;
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

    // Clear back and front buffers
    display.clear_screen().unwrap();

    let background_color = Rgb888::new(0x00, 0xFF, 0xFF);

    // Fill the backbuffer
    display.fill_color(&background_color).unwrap();

    // DMA the backbuffer to the frontbuffer/display
    display.swap_buffers().unwrap();

    let styled_text = Text::new("Hello Rust!", Point::new(100, 100))
        .into_styled(TextStyle::new(Font12x16, Rgb888::BLACK));

    styled_text.draw(&mut display).unwrap();

    display.swap_buffers().unwrap();

    writeln!(serial, "All done").ok();

    loop {
        hal::cortex_a::asm::nop();
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

raspi3_boot::entry!(kernel_entry);
