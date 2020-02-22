#![no_std]
#![no_main]

extern crate bcm2711_hal as hal;

use crate::hal::bcm2711::gpio::GPIO;
use crate::hal::bcm2711::mbox::MBOX;
use crate::hal::bcm2711::sys_timer::SysTimer;
use crate::hal::bcm2711::uart1::UART1;
use crate::hal::clocks::Clocks;
use crate::hal::eth::*;
use crate::hal::mailbox::*;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::time::Bps;
use arr_macro::arr;
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
    let sys_counter = sys_timer.split().sys_counter;

    writeln!(serial, "GENET Ethernet example").ok();

    writeln!(serial, "{:#?}", clocks).ok();

    let mac_addr = EthernetAddress::from(*get_mac_address(&mut mbox).mac_address());
    writeln!(serial, "MAC address: {}", mac_addr).ok();

    let eth_devices = Devices::new();

    // TODO - just putting these massive blobs in the bss for now
    let rx_descriptors = unsafe {
        static mut RX_DESC: Descriptors = arr![Descriptor::zero(); 256];
        &mut RX_DESC[..]
    };

    for (idx, desc) in rx_descriptors.iter().enumerate() {
        writeln!(serial, "rx_desc[{}] {}", idx, desc).ok();
    }

    let mut pkt_buffer = unsafe {
        static mut PKT: [u8; MAX_MTU_SIZE] = [0; MAX_MTU_SIZE];
        &mut PKT[..]
    };

    let mut eth = Eth::new(eth_devices, sys_counter, mac_addr, rx_descriptors).unwrap();

    writeln!(serial, "Ethernet initialized").ok();

    writeln!(serial, "Recv loop").ok();

    let forged_pkt: [u8; 60] = [
        0x3C, 0xE1, 0xA1, 0x4E, 0x48, 0x5C, 0xDC, 0xA6, 0x32, 0x2D, 0xD7, 0x6C, 0x88, 0x74, 0xE2,
        0xE4, 0x36, 0x23, 0xFD, 0xEA, 0xCA, 0x87, 0x49, 0x5B, 0xD0, 0x20, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    loop {
        match eth.recv(&mut pkt_buffer, &mut serial) {
            Ok(size) => {
                if size != 0 {
                    writeln!(serial, "Recv'd {} bytes", size).ok();
                    for b in 0..size {
                        write!(serial, "{:02X} ", pkt_buffer[b]).ok();
                    }
                    write!(serial, "\n").ok().unwrap();

                    //writeln!(serial, "Sending forged pkt {} bytes",
                    // forged_pkt.len()).ok();
                    // eth.send(&forged_pkt).unwrap();
                }
            }
            Err(e) => writeln!(serial, "Eth Error {:?}", e).ok().unwrap(),
        }
    }
}

fn get_mac_address(mbox: &mut Mailbox) -> GetMacAddressRepr {
    let resp = mbox
        .call(Channel::Prop, &GetMacAddressRepr::default())
        .expect("MBox call()");

    if let RespMsg::GetMacAddress(repr) = resp {
        repr
    } else {
        panic!("Invalid response\n{:#?}", resp);
    }
}

raspi3_boot::entry!(kernel_entry);
