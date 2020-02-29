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

fn kernel_entry() -> ! {
    let mut mbox = Mailbox::new(MBOX::new());
    let clocks = Clocks::freeze(&mut mbox).unwrap();
    let gpio = GPIO::new();
    let gp = gpio.split();

    let tx = gp.p14.into_alternate_af5();
    let rx = gp.p15.into_alternate_af5();

    let mut serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200), clocks);

    writeln!(serial, "Embedded graphics example").ok();

    let sn = get_serial_number(&mut mbox).serial_number();
    writeln!(serial, "Serial number: {:#010X}", sn).ok();

    // Construct the DMA peripheral, reset and enable CH0
    let dma = DMA::new();
    let mut dma_parts = dma.split();
    dma_parts.enable.enable.modify(Enable::En0::Set);
    let mut dma_chan = dma_parts.ch0;

    type Buffer = [u64; 8];

    let dma_src_buffer: Buffer = [1, 2, 3, 4, 5, 6, 7, 8];
    let mut dma_dest_buffer: Buffer = [0; 8];

    writeln!(serial, "Before the transfer:").ok();
    writeln!(serial, "dma_src_buffer: {:?}", dma_src_buffer).ok();
    writeln!(serial, "dma_dest_buffer: {:?}", dma_dest_buffer).ok();
    assert_ne!(dma_src_buffer, dma_dest_buffer);

    dma_chan.reset();

    writeln!(serial, "DMA Channel ID: 0x{:X}", dma_chan.id()).ok();

    assert_eq!(dma_src_buffer.len(), dma_dest_buffer.len());
    assert_ne!(dma_src_buffer, dma_dest_buffer);

    let transfer_len = core::mem::size_of::<Buffer>() as u32;
    writeln!(serial, "Transfer length (bytes): {}", transfer_len).ok();

    let src_paddr = dma_src_buffer.as_ptr() as *const _ as u32;
    let dest_paddr = dma_dest_buffer.as_ptr() as *const _ as u32;

    writeln!(serial, "Source PAddr: 0x{:X}", src_paddr).ok();
    writeln!(serial, "Destination PAddr: 0x{:X}", dest_paddr).ok();

    let mut dcb = dma::ControlBlock::new();

    writeln!(serial, "DCB PAddr: 0x{:X}", dcb.as_paddr()).ok();

    dcb.set_length(dma::TransferLength::ModeLinear(transfer_len));
    dcb.set_src(src_paddr);
    dcb.set_src_width(dma::TransferWidth::Bits32);
    dcb.info.set_src_inc(true);
    dcb.set_dest(dest_paddr);
    dcb.set_dest_width(dma::TransferWidth::Bits32);
    dcb.info.set_dest_inc(true);

    dcb.info.set_wait_resp(true);
    dcb.info.set_burst_len(0);

    writeln!(serial, "{}", dcb).ok();

    let txfr_res = dma::TransferResources {
        src_cached: true,
        dest_cached: true,
        dcb: &dcb,
        src_buffer: &dma_src_buffer,
        dest_buffer: &mut dma_dest_buffer,
    };

    assert_eq!(dma_chan.is_busy(), false, "DMA channel is busy before use?");

    dma_chan.start(&txfr_res);

    dma_chan.wait();

    assert_eq!(dma_chan.errors(), false);
    assert_eq!(dma_chan.is_busy(), false);

    writeln!(serial, "dma_src_buffer: {:?}", dma_src_buffer).ok();
    writeln!(serial, "dma_dest_buffer: {:?}", dma_dest_buffer).ok();

    assert_eq!(dma_src_buffer, dma_dest_buffer);

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

raspi3_boot::entry!(kernel_entry);
