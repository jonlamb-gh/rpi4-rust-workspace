#![no_std]
#![no_main]

extern crate bcm2711_hal as hal;

use crate::hal::bcm2711::gpio::GPIO;
use crate::hal::bcm2711::mbox::MBOX;
use crate::hal::bcm2711::sys_timer::SysTimer;
use crate::hal::bcm2711::uart0::UART0;
use crate::hal::clocks::Clocks;
use crate::hal::mailbox::Mailbox;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::time::Bps;
use core::fmt::Write;

fn kernel_entry() -> ! {
    let mut mbox = Mailbox::new(MBOX::new());
    let clocks = Clocks::freeze(&mut mbox).unwrap();
    let gpio = GPIO::new();

    let gp = gpio.split();
    let tx = gp.p14.into_alternate_af0();
    let rx = gp.p15.into_alternate_af0();

    let mut serial = Serial::uart0(UART0::new(), (tx, rx), Bps(115200), clocks);

    let sys_timer = SysTimer::new();
    let mut sys_counter = sys_timer.split().sys_counter;

    writeln!(serial, "{:#?}", clocks).ok();

    loop {
        writeln!(serial, "UART0 example").ok();
        sys_counter.delay_ms(500u32);
    }
}

raspi3_boot::entry!(kernel_entry);
