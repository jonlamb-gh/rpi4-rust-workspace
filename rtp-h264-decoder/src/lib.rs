//! TODO

#![no_std]

//use crate::header::Header;
//use byteorder::{BigEndian, ByteOrder};
pub use crate::nal_unit_type::NalUnitType;
pub use rtp;
//use log::{trace, warn};

pub mod fragmentation_unit;
pub mod header;
pub mod nal_unit_type;

pub const START_SEQ: [u8; 4] = [0x00, 0x00, 0x00, 0x01];

//#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
//pub enum Error {
//    BadFirstPacket,
//    StorageOverflow,
//    TableSize,
//    DroppedSequence,
//    RtpPayloadType(u8),
//    Header(header::Error),
//    Decoder(nanojpeg_rs::Error),
//}
