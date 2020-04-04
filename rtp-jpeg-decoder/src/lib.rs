//! [RFC2435](https://tools.ietf.org/html/rfc2435)

#![no_std]

// TODO -
// module for header, looks like RtpPacket impl
// this decoder will take rtp::Packet's as input
// it will buffer the fragments until a full frame is constructed
// then it runs it through the nanojpg-rs decoder

use crate::header::Header;

pub mod header;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Error {
    RtpPayloadType(u8),
}

/// JPEG payload type
///
/// [RFC189](https://tools.ietf.org/html/rfc1890)
pub const RTP_PAYLOAD_TYPE_JPEG: u8 = 26;
