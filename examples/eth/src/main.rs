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
    let rx_cbs = unsafe {
        static mut RX_CBS: ControlBlocks = arr![ControlBlock::zero(); 256];
        &mut RX_CBS[..]
    };

    let tx_cbs = unsafe {
        static mut TX_CBS: ControlBlocks = arr![ControlBlock::zero(); 256];
        &mut TX_CBS[..]
    };

    let rx_rings = unsafe {
        static mut RX_RINGS: RxRings = arr![RxRing::zero(); 17];
        &mut RX_RINGS[..]
    };

    let tx_rings = unsafe {
        static mut TX_RINGS: TxRings = arr![TxRing::zero(); 17];
        &mut TX_RINGS[..]
    };

    let mut pkt_buffer = unsafe {
        static mut PKT: [u8; MAX_MTU_SIZE] = [0; MAX_MTU_SIZE];
        &mut PKT[..]
    };

    let mut eth = Eth::new(
        eth_devices,
        sys_counter,
        mac_addr,
        rx_cbs,
        tx_cbs,
        rx_rings,
        tx_rings,
    )
    .unwrap();

    writeln!(serial, "Ethernet initialized").ok();

    writeln!(serial, "link up: {}", eth.link_up()).ok();
    writeln!(serial, "link speed: {}", eth.link_speed()).ok();

    // Wait for link to be up
    loop {
        if eth.link_up() {
            break;
        }

        // TODO - wait ~2 seconds
        // update_phy()
    }

    writeln!(serial, "Send loop").ok();

    let forged_pkt: [u8; 60] = [
        0x3C, 0xE1, 0xA1, 0x4E, 0x48, 0x5C, 0xDC, 0xA6, 0x32, 0x2D, 0xD7, 0x6C, 0x88, 0x74, 0xE2,
        0xE4, 0x36, 0x23, 0xFD, 0xEA, 0xCA, 0x87, 0x49, 0x5B, 0xD0, 0x20, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    for _ in 0..10 {
        eth.send(&forged_pkt).unwrap();
    }

    writeln!(serial, "Recv loop").ok();

    loop {
        match eth.recv(&mut pkt_buffer) {
            Ok(size) => {
                if size != 0 {
                    writeln!(serial, "Recv'd {} bytes", size).ok();
                    for b in 0..size {
                        write!(serial, "{:02X} ", pkt_buffer[b]).ok();
                    }
                    write!(serial, "\n").ok().unwrap();
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
