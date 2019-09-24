#![no_std]
#![no_main]

extern crate bcm2711_hal as hal;

use crate::hal::bcm2711::gpio::GPIO;
use crate::hal::bcm2711::mbox::MBOX;
use crate::hal::bcm2711::sys_timer::SysTimer;
use crate::hal::bcm2711::uart1::UART1;
use crate::hal::clocks::Clocks;
use crate::hal::mailbox::*;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::time::Bps;
use core::fmt::Write;

fn kernel_entry() -> ! {
    let mut mbox = Mailbox::new(MBOX::new());
    let clocks = Clocks::freeze(&mut mbox).unwrap();
    let gpio = GPIO::new();
    let gp = gpio.split();

    let tx = gp.p14.into_alternate_af5();
    let rx = gp.p15.into_alternate_af5();

    let mut serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200), clocks);

    let sys_timer = SysTimer::new();
    let mut sys_counter = sys_timer.split().sys_counter;

    writeln!(serial, "Mailbox example").ok();

    writeln!(serial, "{:#?}", clocks).ok();

    let sn = get_serial_number(&mut mbox).serial_number();
    writeln!(serial, "Serial number: {:#010X}", sn).ok();

    let arm_mem = get_arm_mem(&mut mbox);

    writeln!(
        serial,
        "ARM memory\n  address: {:#010X} size: 0x{:X}",
        arm_mem.address(),
        arm_mem.size()
    )
    .ok();

    let vc_mem = get_vc_mem(&mut mbox);

    writeln!(
        serial,
        "VideoCore memory\n  address: {:#010X} size: 0x{:X}",
        vc_mem.address(),
        vc_mem.size()
    )
    .ok();

    writeln!(serial, "Requesting default framebuffer allocation").ok();

    let fb = alloc_framebuffer(&mut mbox);

    writeln!(
        serial,
        "  width: {} height: {}",
        fb.virt_width, fb.virt_height
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

    loop {
        let repr = get_temp(&mut mbox);
        writeln!(serial, "Temp: {}", repr.temp()).ok();

        sys_counter.delay_ms(500u32);
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

fn get_arm_mem(mbox: &mut Mailbox) -> GetArmMemRepr {
    let resp = mbox
        .call(Channel::Prop, &GetArmMemRepr::default())
        .expect("MBox call()");

    if let RespMsg::GetArmMem(repr) = resp {
        repr
    } else {
        panic!("Invalid response\n{:#?}", resp);
    }
}

fn get_vc_mem(mbox: &mut Mailbox) -> GetVcMemRepr {
    let resp = mbox
        .call(Channel::Prop, &GetVcMemRepr::default())
        .expect("MBox call()");

    if let RespMsg::GetVcMem(repr) = resp {
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
