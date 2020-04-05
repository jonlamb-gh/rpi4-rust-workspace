#![no_std]
#![no_main]

extern crate bcm2711_hal as hal;

// TODO - hal prelude should deal with a lot of these
use crate::hal::bcm2711::dma::{Enable, DMA};
use crate::hal::bcm2711::gpio::GPIO;
use crate::hal::bcm2711::mbox::MBOX;
use crate::hal::bcm2711::sys_timer::SysTimer;
use crate::hal::bcm2711::uart1::UART1;
use crate::hal::cache;
use crate::hal::clocks::Clocks;
use crate::hal::dma;
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
use rtp_jpeg_decoder::*;
use smoltcp::iface::{EthernetInterfaceBuilder, NeighborCache, Routes};
use smoltcp::socket::{
    SocketSet, TcpSocket, TcpSocketBuffer, UdpPacketMetadata, UdpSocket, UdpSocketBuffer,
};
use smoltcp::wire::{EthernetAddress, IpCidr, IpEndpoint, Ipv4Address};
use smoltcp_phy::EthDevice;

mod net;

// add config.txt memory split
//
// previously: 0x0010_0000
// new       : 0x0100_0000
//
// TODO - inspect the binary and see how close things are

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

const SRC_IP: [u8; 4] = [192, 168, 1, 72];
const DST_IP: [u8; 4] = [192, 168, 1, 64];

const CLIENT_IP: Ipv4Address = Ipv4Address(DST_IP);
const CLIENT_PORT: u16 = 554;

const UDP_SERVER_IP: Ipv4Address = Ipv4Address(SRC_IP);
const UDP_SERVER_PORT: u16 = 49154;

const HEAP_SIZE: usize = 3 * 720 * 480 * 5;

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
            //.map(|()| log::set_max_level(LevelFilter::Error))
            .unwrap();
    }

    let sys_timer = SysTimer::new();
    let sys_timer_parts = sys_timer.split();
    let mut sys_counter = sys_timer_parts.sys_counter;
    let mut timer = sys_timer_parts.timer1;

    info!("smoltcp IP example");

    info!("{:#?}", clocks);

    // Construct the DMA peripheral, reset and enable CH0
    let dma = DMA::new();
    let mut dma_parts = dma.split();
    dma_parts.enable.enable.modify(Enable::En0::Set);
    let mut dma_chan = dma_parts.ch0;
    dma_chan.reset();
    info!("DMA Channel ID: 0x{:X}", dma_chan.id());

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

    info!("Requesting framebuffer allocation");

    let fb = alloc_framebuffer(&mut mbox);

    info!(
        "width: {} height: {} pitch {} {:?}",
        fb.virt_width,
        fb.virt_height,
        fb.pitch(),
        fb.pixel_order,
    );

    info!(
        "fb address: {:#010X} bus_address: {:#010X} size: 0x{:X}",
        fb.alloc_buffer_address(),
        fb.alloc_buffer_bus_address(),
        fb.alloc_buffer_size()
    );

    assert_eq!(fb.virt_width, WIDTH);
    assert_eq!(fb.virt_height, HEIGHT);

    // TODO - feature for nanojpeg to produce RGB/BGR 32 bpp data so
    // we can DMA it directly instead of copying the RGB24 into BGR32
    let vc_mem_size = fb.alloc_buffer_size();
    let vc_mem_words = vc_mem_size / 4;
    info!("bytes {} - words {}", vc_mem_size, vc_mem_words,);
    let frontbuffer_mem = unsafe {
        core::slice::from_raw_parts_mut(fb.alloc_buffer_address() as *mut u32, vc_mem_words)
    };

    const STATIC_SIZE: usize = WIDTH * HEIGHT * 4;
    assert!(vc_mem_size <= STATIC_SIZE);

    let dcb_mem = unsafe {
        static mut DCB_MEM: [dma::ControlBlock; 1] = [dma::ControlBlock::new()];
        &mut DCB_MEM[..]
    };

    info!("Display initialized");

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

    let mut rx_meta: [UdpPacketMetadata; UDP_NUM_PACKETS] =
        [UdpPacketMetadata::EMPTY; UDP_NUM_PACKETS];
    let mut tx_meta: [UdpPacketMetadata; 4] = [UdpPacketMetadata::EMPTY; 4];
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

    let bbp: usize = 4;
    let frontbuffer_stride = (fb.pitch() - (fb.virt_width * bbp)) as u32;
    let src_stride = 0;

    let transfer_length = dma::TransferLength::Mode2D(
        // Transfer length in bytes of a row
        (bbp * fb.virt_width) as _,
        // How many x-length transfers are performed
        (fb.virt_height - 1) as _,
    );

    // Initialize a DMA control block for the transfer
    let dcb = &mut dcb_mem[0];
    dcb.init();
    dcb.set_length(transfer_length);
    dcb.set_src_width(dma::TransferWidth::Bits128);
    dcb.stride.set_src_stride(src_stride);
    dcb.info.set_src_inc(true);
    dcb.set_dest(frontbuffer_mem.as_ptr() as u32);
    dcb.set_dest_width(dma::TransferWidth::Bits128);
    dcb.stride.set_dest_stride(frontbuffer_stride);
    dcb.info.set_dest_inc(true);
    dcb.info.set_wait_resp(true);
    dcb.info.set_burst_len(4);

    // TODO - hack until I redo the DMA impl
    // src/dst refs are not used
    let unused_src_buffer: [u32; 0] = [];
    let mut unused_dest_buffer: [u32; 0] = [];

    info!("Run loop");

    timer.start(200.hz());

    loop {
        // TODO
        //block!(timer.wait()).unwrap();
        if timer.wait().is_ok() {
            let time = sys_counter.get_time();
            net.poll(time);
        }

        for _ in 0..UDP_NUM_PACKETS {
            net.recv_udp(|data| {
                info!("UDP recvd {} bytes", data.len());

                match rtp::Packet::new_checked(data) {
                    //Err(e) => warn!("rtp::Packet error {:?}", e),
                    Err(e) => panic!("rtp::Packet error {:?}", e),
                    Ok(pkt) => match decoder.decode(&pkt) {
                        Err(e) => warn!("JPEGDecoder error {:?}", e),
                        //Err(e) => panic!("JPEGDecoder error {:?}", e),
                        Ok(maybe_image) => match maybe_image {
                            None => info!("Ok"),
                            Some(image_info) => {
                                info!("{}", image_info);
                                assert_eq!(image_info.image.len() / 4, WIDTH * HEIGHT);

                                dcb.set_src(image_info.image.as_ptr() as u32);

                                unsafe {
                                    cache::clean_data_cache_range(
                                        image_info.image.as_ptr() as _,
                                        image_info.image.len(),
                                    );
                                }

                                let txfr_res = dma::TransferResources {
                                    src_cached: false,
                                    dest_cached: false,
                                    dcb: &dcb,
                                    src_buffer: &unused_src_buffer,
                                    dest_buffer: &mut unused_dest_buffer,
                                };

                                // Wait for DMA to be ready, then do the transfer
                                while dma_chan.is_busy() == true {
                                    hal::cortex_a::asm::nop();
                                }
                                dma_chan.start(&txfr_res);
                                dma_chan.wait();

                                assert!(!dma_chan.errors());
                            }
                        },
                    },
                }
            })
            .unwrap();
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
    let mut req = AllocFramebufferRepr::default();
    req.phy_width = WIDTH;
    req.virt_width = WIDTH;
    req.phy_height = HEIGHT;
    req.virt_height = HEIGHT;
    let resp = mbox.call(Channel::Prop, &req).expect("MBox call()");

    if let RespMsg::AllocFramebuffer(repr) = resp {
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

    //debug!("malloc size {}", size);
    let size = roundup(size, MIN_ALIGN);
    //debug!(" -- aligned size {}", size);
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
    //debug!(" -- WMARK {}", WMARK);
    //debug!(" -- at {:?}", ptr);
    ptr as *mut ctypes::c_void
}

#[no_mangle]
pub unsafe extern "C" fn njFreeMem(ptr: *mut ctypes::c_void) {
    //debug!("free {:?}", ptr);
    if ptr.is_null() {
        return;
    }
    let layout = get_layout(ptr as *mut u8);
    WMARK += layout.size();
    //debug!(" ++ WMARK {}", WMARK);
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
    //debug!("Insert layout {:?} : {:?}", ptr, layout);
    let _ = MAP.insert(ptr as usize, layout).expect("TODO");
}

unsafe fn get_layout(ptr: *mut u8) -> Layout {
    //debug!("Get layout {:?}", ptr);
    MAP.get(&(ptr as usize)).expect("TODO").clone()
}

unsafe fn delete_layout(ptr: *mut u8) {
    //debug!("Delete layout {:?}", ptr);
    let _ = MAP.remove(&(ptr as usize)).expect("TODO");
}

raspi3_boot::entry!(kernel_entry);
