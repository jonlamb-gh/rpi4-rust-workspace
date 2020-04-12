//! [RFC1889](https://tools.ietf.org/html/rfc1889)

#![no_std]

use byteorder::{BigEndian, ByteOrder};
use core::fmt;

/// Minimum of 12 bytes
pub const HEADER_SIZE: usize = 12;

/// Only version supported is V2
pub const VERSION_V2: u8 = 2;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Error {
    Truncated,
    Version,
}

/// RTP packet
///
/// [RFC1889](https://tools.ietf.org/html/rfc1889#section-5.1)
#[derive(Debug, Clone)]
pub struct Packet<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    type Field = ::core::ops::Range<usize>;
    type Rest = ::core::ops::RangeFrom<usize>;

    /// Version (2 bits), padding (1 bit), extension (1 bit) and CSRC count (4
    /// bits)
    pub const BYTE0: usize = 0;
    /// Marker (1 bit) and payload type (7 bits)
    pub const BYTE1: usize = 1;
    /// Sequence number (16 bits)
    pub const SEQNUM: Field = 2..4;
    /// Timestamp (32 bits)
    pub const TIMESTAMP: Field = 4..8;
    /// SSRC (32 bits)
    pub const SSRC: Field = 8..12;
    /// Remaining
    pub const PAYLOAD: Rest = crate::HEADER_SIZE..;
}

impl<T: AsRef<[u8]>> Packet<T> {
    pub fn new_unchecked(buffer: T) -> Packet<T> {
        Packet { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<Packet<T>, Error> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        packet.check_version()?;
        Ok(packet)
    }

    pub fn check_len(&self) -> Result<(), Error> {
        let len = self.buffer.as_ref().len();
        if len < field::PAYLOAD.start {
            Err(Error::Truncated)
        } else {
            Ok(())
        }
    }

    pub fn check_version(&self) -> Result<(), Error> {
        if self.version() != VERSION_V2 {
            Err(Error::Version)
        } else {
            Ok(())
        }
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    // TODO - assumes no extra CSRC list items
    pub fn header_len() -> usize {
        field::PAYLOAD.start
    }

    #[inline]
    pub fn version(&self) -> u8 {
        let data = self.buffer.as_ref();
        (data[field::BYTE0] >> 6) & 0x03
    }

    #[inline]
    pub fn contains_padding(&self) -> bool {
        let data = self.buffer.as_ref();
        (data[field::BYTE0] >> 5) & 0x01 != 0
    }

    #[inline]
    pub fn contains_extension(&self) -> bool {
        let data = self.buffer.as_ref();
        (data[field::BYTE0] >> 4) & 0x01 != 0
    }

    #[inline]
    pub fn csrc_count(&self) -> usize {
        let data = self.buffer.as_ref();
        usize::from(data[field::BYTE0] & 0x0F)
    }

    #[inline]
    pub fn contains_marker(&self) -> bool {
        let data = self.buffer.as_ref();
        (data[field::BYTE1] >> 7) & 0x01 != 0
    }

    #[inline]
    pub fn payload_type(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::BYTE1] & 0x7F
    }

    #[inline]
    pub fn sequence_number(&self) -> u16 {
        let data = self.buffer.as_ref();
        BigEndian::read_u16(&data[field::SEQNUM])
    }

    #[inline]
    pub fn timestamp(&self) -> u32 {
        let data = self.buffer.as_ref();
        BigEndian::read_u32(&data[field::TIMESTAMP])
    }

    #[inline]
    pub fn sync_source(&self) -> u32 {
        let data = self.buffer.as_ref();
        BigEndian::read_u32(&data[field::SSRC])
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Packet<&'a T> {
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        let pad = self.contains_padding();
        let data = self.buffer.as_ref();
        if pad {
            let num_pad_bytes = usize::from(data[data.len() - 1]);
            let tail = data.len() - 1 - num_pad_bytes;
            &data[field::PAYLOAD.start..tail]
        } else {
            &data[field::PAYLOAD]
        }
    }
}

impl<T: AsRef<[u8]>> fmt::Display for Packet<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "RTP Packet {{v={}, p={}, x={}, m={}, pt={}, sn={}, t={}}}",
            self.version(),
            self.contains_padding(),
            self.contains_extension(),
            self.contains_marker(),
            self.payload_type(),
            self.sequence_number(),
            self.timestamp(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // 1 padding byte, trailing 0xFF is discarded
    static PACKET_BYTES: [u8; 24] = [
        0xA0, 0x1A, 0xD7, 0xDD, 0x03, 0x4F, 0xAF, 0xE2, 0x27, 0x13, 0x17, 0xD1, 0x00, 0x00, 0x00,
        0x00, 0x01, 0x3F, 0x50, 0x3C, 0xE3, 0xEE, 0xFF, 0x01,
    ];

    static PAYLOAD_BYTES: [u8; 10] = [0x00, 0x00, 0x00, 0x00, 0x01, 0x3F, 0x50, 0x3C, 0xE3, 0xEE];

    #[test]
    fn header_len() {
        assert_eq!(Packet::<&[u8]>::header_len(), HEADER_SIZE);
    }

    #[test]
    fn deconstruct() {
        let p = Packet::new_checked(&PACKET_BYTES[..]).unwrap();
        assert_eq!(p.version(), VERSION_V2);
        assert_eq!(p.contains_padding(), true);
        assert_eq!(p.contains_extension(), false);
        assert_eq!(p.csrc_count(), 0);
        assert_eq!(p.contains_marker(), false);
        assert_eq!(p.payload_type(), 26);
        assert_eq!(p.sequence_number(), 0xD7DD);
        assert_eq!(p.timestamp(), 0x034F_AFE2);
        assert_eq!(p.sync_source(), 0x2713_17D1);
        assert_eq!(p.payload().len(), PAYLOAD_BYTES.len());
        assert_eq!(p.payload(), &PAYLOAD_BYTES[..]);
    }
}
