// TODO - make a proper example
// RUST_LOG=trace
// cargo run --example rtsp-decode --target x86_64-unknown-linux-gnu

use rtp_jpeg_decoder::*;
use rtsp::*;
use std::convert::TryFrom;
use std::io::prelude::*;
use std::net::UdpSocket;
use std::net::{Shutdown, TcpStream};
use std::str;

fn main() -> std::io::Result<()> {
    env_logger::init();

    let mut frame_buffer = vec![0; 65536];

    let dec = NanoJPeg::init();
    let mut decoder = JPEGDecoder::new(dec, &mut frame_buffer).unwrap();

    println!("Connecting");

    let mut stream = TcpStream::connect("127.0.0.1:554")?;

    let mut session: Option<Session> = None;

    let mut requests = [
        request_for_options(),
        request_for_describe(),
        request_for_setup(),
        request_for_play(),
    ];

    for req in requests.iter_mut() {
        if req.request_line.method == Method::Play {
            req.headers
                .push(session.as_ref().unwrap().clone().into())
                .unwrap();
        }

        let mut tx = String::new();
        req.emit(&mut tx).unwrap();
        let len = tx.len();
        println!("\n\nSending request {} bytes", len);
        println!("{}", req);
        stream.write(&tx.as_bytes()[..len]).unwrap();

        std::thread::sleep(std::time::Duration::from_millis(250));

        println!("Reading...");
        let mut rx = [0 as u8; 1024];
        match stream.read(&mut rx) {
            Ok(size) => {
                println!("Rx {} bytes", size);
                let text = str::from_utf8(&rx).unwrap();
                println!("---\n{}\n---", text);
                let resp = Response::parse(text).unwrap().1;
                println!("Resp {}", resp);

                if let Some(s) = resp.headers.session() {
                    println!("Found session: {}", s);

                    if session.is_none() {
                        session = Some(s.clone().into());
                    }
                }
            }
            Err(_) => {
                println!(
                    "An error occurred, terminating connection with {}",
                    stream.peer_addr().unwrap()
                );
                stream.shutdown(Shutdown::Both).unwrap();
                todo!();
            }
        }
    }

    println!("UDP listen on port 49154");
    let socket = UdpSocket::bind("0.0.0.0:49154")?;
    let mut loop_counter = 0;
    let mut rx = [0 as u8; 1500];
    loop {
        match socket.recv_from(&mut rx) {
            Ok((amt, src)) => {
                println!("Rx UDP {} bytes from {}", amt, src);

                match rtp::Packet::new_checked(&rx[..amt]) {
                    Err(e) => panic!("rtp::Packet error {:?}", e),
                    Ok(pkt) => match decoder.decode(&pkt) {
                        Err(e) => panic!("JPEGDecoder error {:?}", e),
                        Ok(maybe_image) => match maybe_image {
                            None => println!("Ok"),
                            Some(image_info) => println!("{}", image_info),
                        },
                    },
                }
            }
            Err(e) => {
                panic!("UDP error {:?}", e);
            }
        }

        loop_counter += 1;
        if loop_counter >= 1000 {
            break;
        }
    }

    Ok(())
}

fn request_for_options() -> Request {
    Request {
        request_line: (
            Method::Options,
            Uri::from("rtsp://127.0.0.1:554/streaming/channels/2"),
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
            Uri::from("rtsp://127.0.0.1:554/streaming/channels/2"),
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
            Uri::from("rtsp://127.0.0.1:554/streaming/channels/2/trackID=1"),
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
            Uri::from("rtsp://127.0.0.1:554/streaming/channels/2"),
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
