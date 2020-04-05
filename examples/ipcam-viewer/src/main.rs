#![no_std]
#![no_main]

extern crate bcm2711_hal as hal;

use crate::hal::bcm2711::gpio::GPIO;
use crate::hal::bcm2711::mbox::MBOX;
use crate::hal::bcm2711::sys_timer::SysTimer;
use crate::hal::bcm2711::uart1::UART1;
use crate::hal::cache;
use crate::hal::clocks::Clocks;
use crate::hal::eth::{self, Eth};
use crate::hal::gpio::{Alternate, Pin14, Pin15, AF5};
use crate::hal::mailbox::*;
use crate::hal::prelude::*;
use crate::hal::serial::Serial;
use crate::hal::time::Bps;
use crate::net::Net;
use crate::net::*;
use arr_macro::arr;
use core::alloc::Layout;
use core::cell::UnsafeCell;
use core::fmt::Write;
use core::ptr;
use core::slice;
use heapless::consts::U256;
use heapless::LinearMap;
use linked_list_allocator::Heap;
use log::{debug, info, warn, LevelFilter, Metadata, Record};
use nb::block;
use rtp_jpeg_decoder::*;
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache, Routes};
use smoltcp::socket::{
    SocketSet, TcpSocket, TcpSocketBuffer, UdpPacketMetadata, UdpSocket, UdpSocketBuffer,
};
use smoltcp::wire::{EthernetAddress, IpCidr, IpEndpoint, Ipv4Address};
use smoltcp_phy::EthDevice;

mod net;

// add config.txt memory split
// previously: 0x0010_0000
// new       : 0x0100_0000

const SRC_IP: [u8; 4] = [192, 168, 1, 72];
const DST_IP: [u8; 4] = [192, 168, 1, 64];

const CLIENT_IP: Ipv4Address = Ipv4Address(DST_IP);
const CLIENT_PORT: u16 = 554;

const UDP_SERVER_IP: Ipv4Address = Ipv4Address(SRC_IP);
const UDP_SERVER_PORT: u16 = 49154;

const HEAP_SIZE: usize = 5 * 720 * 480;

static GLOBAL_LOGGER: SerialLogger = SerialLogger::new();
static mut MAP: LinearMap<usize, Layout, U256> = LinearMap(heapless::i::LinearMap::new());
static mut HEAP_MEM: [u8; HEAP_SIZE] = [0; HEAP_SIZE];
static mut HEAP: Heap = Heap::empty();
static mut WMARK: usize = HEAP_SIZE;

#[allow(non_camel_case_types)]
mod ctypes {
    pub type size_t = usize;
    pub type c_int = i32;
    pub type c_void = core::ffi::c_void;
    pub type c_uchar = u8;
}

type LogInner = Serial<UART1, (Pin14<Alternate<AF5>>, Pin15<Alternate<AF5>>)>;

struct SerialLogger {
    serial: UnsafeCell<Option<LogInner>>,
}

impl SerialLogger {
    pub const fn new() -> SerialLogger {
        SerialLogger {
            serial: UnsafeCell::new(None),
        }
    }

    pub fn set_inner(&self, inner: LogInner) {
        let serial = unsafe { &mut *self.serial.get() };
        let _ = serial.replace(inner);
    }
}

unsafe impl Sync for SerialLogger {}

impl log::Log for SerialLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let maybe_serial = unsafe { &mut *self.serial.get() };
            if let Some(serial) = maybe_serial {
                writeln!(serial, "[{}] {}", record.level(), record.args()).unwrap();
            } else {
                panic!("Logger was used before being given its inner type");
            }
        }
    }

    fn flush(&self) {}
}

fn kernel_entry() -> ! {
    let mut mbox = Mailbox::new(MBOX::new());
    let clocks = Clocks::freeze(&mut mbox).unwrap();
    let gpio = GPIO::new();

    let gp = gpio.split();
    let tx = gp.p14.into_alternate_af5();
    let rx = gp.p15.into_alternate_af5();

    let serial = Serial::uart1(UART1::new(), (tx, rx), Bps(115200), clocks);

    GLOBAL_LOGGER.set_inner(serial);
    unsafe {
        log::set_logger_racy(&GLOBAL_LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .unwrap();
    }

    let sys_timer = SysTimer::new();
    let sys_timer_parts = sys_timer.split();
    let mut sys_counter = sys_timer_parts.sys_counter;
    let mut timer = sys_timer_parts.timer1;

    info!("smoltcp IP example");

    info!("{:#?}", clocks);

    let arm_mem = get_arm_mem(&mut mbox);
    info!(
        "ARM memory\n  address: {:#010X} size: 0x{:X}",
        arm_mem.address(),
        arm_mem.size()
    );

    let vc_mem = get_vc_mem(&mut mbox);
    info!(
        "VideoCore memory\n  address: {:#010X} size: 0x{:X}",
        vc_mem.address(),
        vc_mem.size()
    );

    let ethernet_addr = EthernetAddress::from_bytes(get_mac_address(&mut mbox).mac_address());
    info!("MAC address: {}", ethernet_addr);

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

    info!("Ethernet initialized");

    info!("Waiting for link-up");

    loop {
        let status = eth.status().unwrap();
        if status.link_status {
            info!("Link is up");
            info!("Speed: {}", status.speed);
            info!("Full duplex: {}", status.full_duplex);

            assert_ne!(status.speed, 0, "Speed is 0");
            assert_eq!(status.full_duplex, true, "Not full duplex");
            break;
        }

        sys_counter.delay_ms(100_u32);
        info!(".");
    }

    let eth_dev = EthDevice { eth };

    let ip = Ipv4Address::from_bytes(&SRC_IP);
    let ip_addr = IpCidr::new(ip.into(), 24);
    let mut ip_addrs = [ip_addr];
    let mut neighbor_storage = [None; NEIGHBOR_CACHE_SIZE];
    let mut routes_storage = [None; ROUTES_SIZE];
    let routes = Routes::new(&mut routes_storage[..]);
    let neighbor_cache = NeighborCache::new(&mut neighbor_storage[..]);
    let iface = EthernetInterfaceBuilder::new(eth_dev)
        .ethernet_addr(ethernet_addr)
        .ip_addrs(&mut ip_addrs[..])
        .routes(routes)
        .neighbor_cache(neighbor_cache)
        .finalize();

    let tcp_client_socket = {
        static mut TCP_RX_DATA: [u8; TCP_SOCKET_BUFFER_SIZE] = [0; TCP_SOCKET_BUFFER_SIZE];
        static mut TCP_TX_DATA: [u8; TCP_SOCKET_BUFFER_SIZE] = [0; TCP_SOCKET_BUFFER_SIZE];
        let tcp_rx_buffer = TcpSocketBuffer::new(unsafe { &mut TCP_RX_DATA[..] });
        let tcp_tx_buffer = TcpSocketBuffer::new(unsafe { &mut TCP_TX_DATA[..] });
        TcpSocket::new(tcp_rx_buffer, tcp_tx_buffer)
    };

    let mut rx_meta = [UdpPacketMetadata::EMPTY];
    let mut tx_meta = [UdpPacketMetadata::EMPTY];
    let udp_server_socket = {
        static mut UDP_RX_DATA: [u8; UDP_SOCKET_BUFFER_SIZE] = [0; UDP_SOCKET_BUFFER_SIZE];
        static mut UDP_TX_DATA: [u8; UDP_SOCKET_BUFFER_SIZE] = [0; UDP_SOCKET_BUFFER_SIZE];
        UdpSocket::new(
            UdpSocketBuffer::new(&mut rx_meta[..], unsafe { &mut UDP_RX_DATA[..] }),
            UdpSocketBuffer::new(&mut tx_meta[..], unsafe { &mut UDP_TX_DATA[..] }),
        )
    };

    let mut sockets_storage = [None, None];
    let mut sockets = SocketSet::new(&mut sockets_storage[..]);

    let tcp_client_handle = sockets.add(tcp_client_socket);
    let udp_server_handle = sockets.add(udp_server_socket);

    let tcp_endpoint = IpEndpoint::new(CLIENT_IP.into(), CLIENT_PORT);
    let udp_endpoint = IpEndpoint::new(UDP_SERVER_IP.into(), UDP_SERVER_PORT);

    info!("IP stack initialized");
    info!("IP address: {}", ip);

    let rtsp_string = rtsp_string();

    let mut net = Net::new(
        iface,
        sockets,
        tcp_client_handle,
        tcp_endpoint,
        udp_server_handle,
        udp_endpoint,
        rtsp_string,
    )
    .unwrap();

    unsafe {
        HEAP.init(HEAP_MEM.as_ptr() as _, HEAP_SIZE);
    }

    info!("Heap init - size {} bytes", HEAP_SIZE);

    let fragment_storage = unsafe {
        static mut FRAG_STORAGE: [u8; 65536] = [0; 65536];
        &mut FRAG_STORAGE[..]
    };

    let dec = NanoJPeg::init();
    let mut decoder = JPEGDecoder::new(dec, fragment_storage).unwrap();

    info!("Run loop");

    timer.start(400.hz());

    // TODO - seem to be dropping rx packets
    // try reducing framerate
    // might be in the eth driver impl...

    loop {
        block!(timer.wait()).unwrap();
        let time = sys_counter.get_time();

        net.poll(time);

        net.recv_udp(|data| {
            info!("UDP recvd {} bytes", data.len());

            match rtp::Packet::new_checked(data) {
                //Err(e) => warn!("rtp::Packet error {:?}", e),
                Err(e) => panic!("rtp::Packet error {:?}", e),
                Ok(pkt) => match decoder.decode(&pkt) {
                    //Err(e) => warn!("JPEGDecoder error {:?}", e),
                    Err(e) => panic!("JPEGDecoder error {:?}", e),
                    Ok(maybe_image) => match maybe_image {
                        None => info!("Ok"),
                        //Some(image_info) => info!("{}", image_info),
                        Some(image_info) => panic!("{}", image_info),
                    },
                },
            }
        })
        .unwrap();
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

#[no_mangle]
pub unsafe extern "C" fn njAllocMem(size: ctypes::c_int) -> *mut ctypes::c_void {
    // TODO
    // https://github.com/ezrosent/allocators-rs/blob/master/malloc-bind/src/lib.rs#L618
    // https://github.com/ezrosent/allocators-rs/blob/master/malloc-bind/src/lib.rs#L184

    let size = size as usize;
    if size == 0 {
        return ptr::null_mut();
    }

    debug!("malloc size {}", size);
    let size = roundup(size, MIN_ALIGN);
    debug!(" -- aligned size {}", size);
    let layout = layout_from_size_align(size as usize, MIN_ALIGN);
    let ptr = HEAP
        .allocate_first_fit(layout.clone())
        .ok()
        .map_or(0 as *mut u8, |allocation| allocation.as_ptr());
    if !ptr.is_null() {
        WMARK -= layout.size();
        insert_layout(ptr, layout);
        cache::clean_and_invalidate_data_cache_range(ptr as _, size);
    } else {
        panic!("Heap out of memory, WMARK = {}", WMARK);
    }
    debug!(" -- WMARK {}", WMARK);
    debug!(" -- at {:?}", ptr);
    ptr as *mut ctypes::c_void
}

#[no_mangle]
pub unsafe extern "C" fn njFreeMem(ptr: *mut ctypes::c_void) {
    debug!("free {:?}", ptr);
    if ptr.is_null() {
        return;
    }
    let layout = get_layout(ptr as *mut u8);
    WMARK += layout.size();
    debug!(" ++ WMARK {}", WMARK);
    delete_layout(ptr as *mut u8);
    HEAP.deallocate(ptr::NonNull::new_unchecked(ptr as *mut u8), layout);
}

#[no_mangle]
pub unsafe extern "C" fn njFillMem(
    block: *mut ctypes::c_void,
    byte: ctypes::c_uchar,
    size: ctypes::c_int,
) {
    if size > 0 {
        let slice = slice::from_raw_parts_mut(block as *mut u8, size as usize);
        slice.iter_mut().for_each(|b| *b = byte);
    }
}

#[no_mangle]
pub unsafe extern "C" fn njCopyMem(
    dst: *mut ctypes::c_void,
    src: *const ctypes::c_void,
    size: ctypes::c_int,
) {
    if size > 0 {
        let dst = slice::from_raw_parts_mut(dst as *mut u8, size as usize);
        let src = slice::from_raw_parts(src as *mut u8, size as usize);
        dst.copy_from_slice(src);
    }
}

const MIN_ALIGN: ctypes::size_t = 8;
//const MIN_ALIGN: ctypes::size_t = 16;

#[inline(always)]
fn roundup(n: ctypes::size_t, multiple: ctypes::size_t) -> ctypes::size_t {
    if n == 0 {
        return multiple;
    }
    let remainder = n % multiple;
    if remainder == 0 {
        n
    } else {
        n + multiple - remainder
    }
}

#[inline(always)]
unsafe fn layout_from_size_align(size: usize, align: usize) -> Layout {
    // TODO - flatten this
    if cfg!(debug_assertions) {
        Layout::from_size_align(size as usize, align).unwrap()
    } else {
        Layout::from_size_align_unchecked(size as usize, align)
    }
}

unsafe fn insert_layout(ptr: *mut u8, layout: Layout) {
    debug!("Insert layout {:?} : {:?}", ptr, layout);
    let _ = MAP.insert(ptr as usize, layout).expect("TODO");
}

unsafe fn get_layout(ptr: *mut u8) -> Layout {
    debug!("Get layout {:?}", ptr);
    MAP.get(&(ptr as usize)).expect("TODO").clone()
}

unsafe fn delete_layout(ptr: *mut u8) {
    debug!("Delete layout {:?}", ptr);
    let _ = MAP.remove(&(ptr as usize)).expect("TODO");
}

raspi3_boot::entry!(kernel_entry);
