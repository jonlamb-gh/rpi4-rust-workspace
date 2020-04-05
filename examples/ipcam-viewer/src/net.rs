use crate::hal::bcm2711::genet::NUM_DMA_DESC;
use crate::hal::eth::MAX_MTU_SIZE;
use crate::hal::time::Instant;
use core::convert::TryFrom;
use core::str;
use heapless::{consts::U512, String};
use log::{debug, error, warn};
use rtsp::*;
use smoltcp::iface::EthernetInterface;
use smoltcp::socket::{SocketHandle, SocketSet, TcpSocket, TcpState, UdpSocket};
use smoltcp::wire::IpEndpoint;
use smoltcp::Error;
use smoltcp_phy::EthDevice;

pub const NEIGHBOR_CACHE_SIZE: usize = 32;
pub const ROUTES_SIZE: usize = 4;
pub const TCP_SOCKET_BUFFER_SIZE: usize = 1024;
// NUM_DMA_DESC * MAX_MTU_SIZE = 256 * 1536 = 393,216
pub const UDP_NUM_PACKETS: usize = NUM_DMA_DESC + 64;
pub const UDP_SOCKET_BUFFER_SIZE: usize = UDP_NUM_PACKETS * MAX_MTU_SIZE;

const TCP_TIMEOUT_DURATION: Option<smoltcp::time::Duration> =
    Some(smoltcp::time::Duration { millis: 5 * 1000 });
const TCP_KEEP_ALIVE_INTERVAL: Option<smoltcp::time::Duration> =
    Some(smoltcp::time::Duration { millis: 2 * 1000 });

// 49152..=65535
const EPHEMERAL_PORT: u16 = 49152;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum RtspState {
    WaitForResponse(Method),
    RequestOptions,
    RequestDescribe,
    RequestSetup,
    RequestPlay,
}

pub type RtspString = String<U512>;

pub const fn rtsp_string() -> RtspString {
    String(heapless::i::String::new())
}

pub struct Net<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx> {
    iface: EthernetInterface<'a, 'b, 'c, EthDevice<'rx, 'tx>>,
    sockets: SocketSet<'d, 'e, 'f>,
    udp_handle: SocketHandle,
    udp_endpoint: IpEndpoint,
    tcp_handle: SocketHandle,
    tcp_endpoint: IpEndpoint,
    tcp_was_connected: bool,
    rtsp_state: RtspState,
    rtsp_string: RtspString,
    session: Option<Session>,
}

impl<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx> Net<'a, 'b, 'c, 'd, 'e, 'f, 'rx, 'tx> {
    pub fn new(
        iface: EthernetInterface<'a, 'b, 'c, EthDevice<'rx, 'tx>>,
        sockets: SocketSet<'d, 'e, 'f>,
        tcp_handle: SocketHandle,
        tcp_endpoint: IpEndpoint,
        udp_handle: SocketHandle,
        udp_endpoint: IpEndpoint,
        rtsp_string: RtspString,
    ) -> Result<Self, Error> {
        let mut eth = Net {
            iface,
            sockets,
            udp_handle,
            udp_endpoint,
            tcp_handle,
            tcp_endpoint,
            tcp_was_connected: true,
            rtsp_state: RtspState::RequestOptions,
            rtsp_string,
            session: None,
        };

        debug!("UDP endpoint {}", eth.udp_endpoint);
        debug!("TCP endpoint {}", eth.tcp_endpoint);

        eth.udp_bind();

        Ok(eth)
    }

    // TODO - check for packet here?
    pub fn recv_udp<F: FnOnce(&[u8])>(&mut self, f: F) -> Result<(), Error> {
        let mut socket = self.sockets.get::<UdpSocket>(self.udp_handle);
        if socket.can_recv() {
            let (data, _remote) = socket.recv()?;
            f(data);
            Ok(())
        } else {
            Ok(())
        }
    }

    // TODO - rate limit the TCP reconnect
    // log a warning if a buffer was exhausted
    pub fn poll(&mut self, time: Instant) {
        let mut reconnect = false;
        let mut tcp_state = TcpState::Closed;

        let t = smoltcp::time::Instant::from_millis(time.total_millis() as i64);
        match self.iface.poll(&mut self.sockets, t) {
            Ok(true) => {
                // Something happened, manage the sockets
                let mut tcp_socket = self.sockets.get::<TcpSocket>(self.tcp_handle);

                let remote_disconnected = if tcp_socket.is_active()
                    && self.tcp_was_connected
                    && (tcp_socket.state() == TcpState::CloseWait)
                {
                    true
                } else {
                    false
                };

                if tcp_socket.is_active() && !self.tcp_was_connected {
                    debug!("TCP connected, state {}", tcp_socket.state());
                    self.tcp_was_connected = true
                } else if (!tcp_socket.is_active() && self.tcp_was_connected) || remote_disconnected
                {
                    debug!("TCP disconnected");
                    self.tcp_was_connected = false;
                    if remote_disconnected {
                        tcp_socket.close();
                    }
                    tcp_socket.abort();
                    reconnect = true;
                }

                tcp_state = tcp_socket.state();
            }
            Err(e) => match e {
                //Error::Exhausted => error!("Socket buffer exhausted"),
                Error::Exhausted => panic!("Socket buffer exhausted"),
                Error::Dropped => warn!("Packet dropped"),
                _ => (),
            },
            _ => (),
        }

        // TODO - need to manage the TCP connection
        // TODO - clean this up
        // - do a teardown first to cleanup?
        if tcp_state == TcpState::Established {
            let mut tcp_socket = self.sockets.get::<TcpSocket>(self.tcp_handle);

            match self.rtsp_state {
                RtspState::WaitForResponse(method) => {
                    //debug!("Wait for response");
                    let mut session = None;
                    if tcp_socket.can_recv() {
                        let mut got_something = false;
                        tcp_socket
                            .recv(|data| {
                                debug!("TCP recv'd {} bytes", data.len());
                                if let Ok(resp_str) = str::from_utf8(data) {
                                    if let Ok((_, resp)) = Response::parse(resp_str) {
                                        debug!("{}", resp);
                                        got_something = true;
                                        if let Some(s) = resp.headers.session() {
                                            debug!("Found session: {}", s);

                                            if session.is_none() {
                                                session = Some(s.clone().into());
                                            }
                                        }
                                    }
                                }
                                (data.len(), ())
                            })
                            .expect("TCP can't recv");

                        if got_something {
                            if session.is_some() {
                                self.session = session;
                            }

                            match method {
                                Method::Options => self.rtsp_state = RtspState::RequestDescribe,
                                Method::Describe => self.rtsp_state = RtspState::RequestSetup,
                                Method::Setup => self.rtsp_state = RtspState::RequestPlay,
                                Method::Play => {
                                    debug!("Got PLAY response, should be streaming now")
                                }
                                _ => (),
                            }
                        }
                    }
                }
                RtspState::RequestOptions => {
                    let req = request_for_options();
                    self.rtsp_string.clear();
                    req.emit(&mut self.rtsp_string).expect("Request emit");
                    let req_size = self.rtsp_string.as_bytes().len();
                    let req_slice = self.rtsp_string.as_bytes();

                    tcp_socket
                        .send(|buffer| {
                            debug!("Sending OPTIONS request {} bytes", req_size);
                            &mut buffer[..req_size].copy_from_slice(req_slice);
                            (req_size, ())
                        })
                        .expect("TCP can't send");

                    self.rtsp_state = RtspState::WaitForResponse(Method::Options);
                }
                RtspState::RequestDescribe => {
                    let req = request_for_describe();
                    self.rtsp_string.clear();
                    req.emit(&mut self.rtsp_string).expect("Request emit");
                    let req_size = self.rtsp_string.as_bytes().len();
                    let req_slice = self.rtsp_string.as_bytes();

                    tcp_socket
                        .send(|buffer| {
                            debug!("Sending DESCRIBE request {} bytes", req_size);
                            &mut buffer[..req_size].copy_from_slice(req_slice);
                            (req_size, ())
                        })
                        .expect("TCP can't send");

                    self.rtsp_state = RtspState::WaitForResponse(Method::Describe);
                }
                RtspState::RequestSetup => {
                    let req = request_for_setup();
                    self.rtsp_string.clear();
                    req.emit(&mut self.rtsp_string).expect("Request emit");
                    let req_size = self.rtsp_string.as_bytes().len();
                    let req_slice = self.rtsp_string.as_bytes();

                    tcp_socket
                        .send(|buffer| {
                            debug!("Sending SETUP request {} bytes", req_size);
                            &mut buffer[..req_size].copy_from_slice(req_slice);
                            (req_size, ())
                        })
                        .expect("TCP can't send");

                    self.rtsp_state = RtspState::WaitForResponse(Method::Setup);
                }
                RtspState::RequestPlay => {
                    let mut req = request_for_play();
                    req.headers
                        .push(self.session.as_ref().unwrap().clone().into())
                        .unwrap();

                    self.rtsp_string.clear();
                    req.emit(&mut self.rtsp_string).expect("Request emit");
                    let req_size = self.rtsp_string.as_bytes().len();
                    let req_slice = self.rtsp_string.as_bytes();

                    tcp_socket
                        .send(|buffer| {
                            debug!("Sending PLAY request {} bytes", req_size);
                            &mut buffer[..req_size].copy_from_slice(req_slice);
                            (req_size, ())
                        })
                        .expect("TCP can't send");

                    self.rtsp_state = RtspState::WaitForResponse(Method::Play);
                }
            }
        }

        if reconnect {
            self.tcp_connect();
        }
    }

    fn tcp_connect(&mut self) {
        let mut socket = self.sockets.get::<TcpSocket>(self.tcp_handle);
        socket.abort();
        debug!("TCP endpoint connecting to {}", self.tcp_endpoint);
        socket
            .connect(self.tcp_endpoint, EPHEMERAL_PORT)
            .expect("TCP socket already open");
        socket.set_timeout(TCP_TIMEOUT_DURATION);
        socket.set_keep_alive(TCP_KEEP_ALIVE_INTERVAL);
    }

    fn udp_bind(&mut self) {
        let mut socket = self.sockets.get::<UdpSocket>(self.udp_handle);
        debug!("UDP endpoint binding on port {}", self.udp_endpoint.port);
        socket
            .bind(self.udp_endpoint)
            .expect("UDP socket already open");
    }
}

fn request_for_options() -> Request {
    Request {
        request_line: (
            Method::Options,
            Uri::from("rtsp://192.168.1.64:554/streaming/channels/2"),
            Version::new(1, 0),
        )
            .into(),
        headers: Headers(
            [CSeq::try_from(1_u32).unwrap().into()]
                .iter()
                .cloned()
                .collect(),
        ),
    }
}

fn request_for_describe() -> Request {
    Request {
        request_line: (
            Method::Describe,
            Uri::from("rtsp://192.168.1.64:554/streaming/channels/2"),
            Version::new(1, 0),
        )
            .into(),
        headers: Headers(
            [
                CSeq::try_from(2_u32).unwrap().into(),
                ("Accept", "application/sdp").into(),
            ]
            .iter()
            .cloned()
            .collect(),
        ),
    }
}

fn request_for_setup() -> Request {
    Request {
        request_line: (
            Method::Setup,
            Uri::from("rtsp://192.168.1.64:554/streaming/channels/2/trackID=1"),
            Version::new(1, 0),
        )
            .into(),
        headers: Headers(
            [
                CSeq::try_from(3_u32).unwrap().into(),
                ("Transport", "RTP/AVP;unicast;client_port=49154-49155").into(),
            ]
            .iter()
            .cloned()
            .collect(),
        ),
    }
}

fn request_for_play() -> Request {
    Request {
        request_line: (
            Method::Play,
            Uri::from("rtsp://192.168.1.64:554/streaming/channels/2"),
            Version::new(1, 0),
        )
            .into(),
        headers: Headers(
            [CSeq::try_from(4_u32).unwrap().into()]
                .iter()
                .cloned()
                .collect(),
        ),
    }
}
