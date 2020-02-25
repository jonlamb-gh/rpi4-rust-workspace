#![no_std]
#![no_main]

extern crate bcm2711_hal as hal;

use crate::hal::bcm2711::gpio::GPIO;
use crate::hal::bcm2711::mbox::MBOX;
use crate::hal::bcm2711::sys_timer::SysTimer;
use crate::hal::bcm2711::uart1::UART1;
use crate::hal::clocks::Clocks;
use crate::hal::eth::{self, Eth};
use crate::hal::mailbox::*;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::time::Bps;
use crate::smoltcp_phy::EthDevice;
use arr_macro::arr;
use core::fmt::Write;
use nb::block;
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache, Routes};
use smoltcp::socket::{SocketSet, TcpSocket, TcpSocketBuffer};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, IpAddress, IpCidr, Ipv4Address};

mod smoltcp_phy;

fn kernel_entry() -> ! {
    let mut mbox = Mailbox::new(MBOX::new());
    let clocks = Clocks::freeze(&mut mbox).unwrap();
    let gpio = GPIO::new();

    let gp = gpio.split();
    let tx = gp.p14.into_alternate_af5();
    let rx = gp.p15.into_alternate_af5();

    let mut serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200), clocks);

    let sys_timer = SysTimer::new();
    let sys_timer_parts = sys_timer.split();
    let mut sys_counter = sys_timer_parts.sys_counter;
    let mut timer = sys_timer_parts.timer1;

    writeln!(serial, "smoltcp IP example").ok();

    writeln!(serial, "{:#?}", clocks).ok();

    let ethernet_addr = EthernetAddress::from_bytes(get_mac_address(&mut mbox).mac_address());
    writeln!(serial, "MAC address: {}", ethernet_addr).ok();

    let eth_devices = eth::Devices::new();

    let rx_descriptors = unsafe {
        static mut RX_DESC: eth::Descriptors = arr![eth::Descriptor::zero(); 256];
        &mut RX_DESC[..]
    };

    let tx_descriptors = unsafe {
        static mut TX_DESC: eth::Descriptors = arr![eth::Descriptor::zero(); 256];
        &mut TX_DESC[..]
    };

    let mut eth = Eth::new(
        eth_devices,
        &mut sys_counter,
        ethernet_addr.0.into(),
        rx_descriptors,
        tx_descriptors,
    )
    .unwrap();

    writeln!(serial, "Ethernet initialized").ok();

    writeln!(serial, "Waiting for link-up").ok();

    loop {
        let status = eth.status().unwrap();
        if status.link_status {
            writeln!(serial, "Link is up").ok();
            writeln!(serial, "Speed: {}", status.speed).ok();
            writeln!(serial, "Full duplex: {}", status.full_duplex).ok();

            assert_ne!(status.speed, 0, "Speed is 0");
            assert_eq!(status.full_duplex, true, "Not full duplex");
            break;
        }

        sys_counter.delay_ms(100_u32);
        writeln!(serial, ".").ok();
    }

    let eth_dev = EthDevice { eth };

    let local_addr = Ipv4Address::new(192, 168, 1, 72);
    let ip_addr = IpCidr::new(IpAddress::from(local_addr), 24);
    let mut ip_addrs = [ip_addr];
    let mut neighbor_storage = [None; 4];
    let mut routes_storage = [None; 2];
    let routes = Routes::new(&mut routes_storage[..]);
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
    let mut iface = EthernetInterfaceBuilder::new(eth_dev)
        .ethernet_addr(ethernet_addr)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .neighbor_cache(neighbor_cache)
        .finalize();

    let server_socket = {
        static mut TCP_SERVER_RX_DATA: [u8; 512] = [0; 512];
        static mut TCP_SERVER_TX_DATA: [u8; 512] = [0; 512];
        let tcp_rx_buffer = TcpSocketBuffer::new(unsafe { &mut TCP_SERVER_RX_DATA[..] });
        let tcp_tx_buffer = TcpSocketBuffer::new(unsafe { &mut TCP_SERVER_TX_DATA[..] });
        TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer)
    };

    let mut sockets_storage = [None];
    let mut sockets = SocketSet::new(&mut sockets_storage[..]);
    let server_handle = sockets.add(server_socket);

    writeln!(serial, "IP stack initialized").ok();
    writeln!(serial, "IP address: {}", local_addr).ok();

    writeln!(serial, "Run loop").ok();

    timer.start(200.hz());

    loop {
        block!(timer.wait()).unwrap();
        let time = Instant::from_millis(sys_counter.read() as i64);

        match iface.poll(&mut sockets, time) {
            Ok(true) => {
                let mut socket = sockets.get::<TcpSocket>(server_handle);
                if !socket.is_open() {
                    socket
                        .listen(80)
                        .or_else(|e| writeln!(serial, "TCP listen error: {:?}", e))
                        .unwrap();
                    writeln!(serial, "Listening on port {}", 80).ok();
                }

                if socket.can_send() {
                    write!(socket, "hello\n")
                        .map(|_| {
                            socket.close();
                        })
                        .or_else(|e| writeln!(serial, "TCP send error: {:?}", e))
                        .unwrap();
                }
            }
            Ok(false) => (),
            Err(e) =>
            // Ignore malformed packets
            {
                writeln!(serial, "Error: {:?}", e).unwrap()
            }
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
