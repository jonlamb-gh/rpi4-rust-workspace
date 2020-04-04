//! [RFC2435](https://tools.ietf.org/html/rfc2435#section-3.1)
//!
//! NOTE: doesn't handle restart marker headers or quantization table headers
//! type <= 63
//! Q value <= 127

use byteorder::{BigEndian, ByteOrder};

/// First 8 bytes are called the "main JPEG header"
pub const MAIN_SIZE: usize = 8;

/// Width and height are encoded in 8-pixel multiples.
/// The maximum width is 2040 pixels.
/// The maximum height is 2040 pixels.
pub const WIDTH_HEIGHT_MULT: u16 = 8;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Error {
    Truncated,
    Type,
    QValue,
}

/// JPEG header
///
/// [RFC2435](https://tools.ietf.org/html/rfc2435#section-3.1)
#[derive(Debug, Clone)]
pub struct Header<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    type Field = ::core::ops::Range<usize>;
    type Rest = ::core::ops::RangeFrom<usize>;

    /// Type specific (8 bits)
    pub const TYPE_SPEC: usize = 0;
    /// Fragment offset (24 bits)
    pub const FRAG_OFFSET: Field = 1..4;
    /// Type (8 bits)
    pub const TYPE: usize = 4;
    /// Q value (8 bits)
    pub const QVALUE: usize = 5;
    /// Width (8 bits)
    pub const WIDTH: usize = 6;
    /// Height (8 bits)
    pub const HEIGHT: usize = 7;
    /// Remaining
    pub const PAYLOAD: Rest = super::MAIN_SIZE..;
}

impl<T: AsRef<[u8]>> Header<T> {
    pub fn new_unchecked(buffer: T) -> Header<T> {
        Header { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<Header<T>, Error> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        packet.check_type()?;
        packet.check_qvalue()?;
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

    pub fn check_type(&self) -> Result<(), Error> {
        // Doesn't handle restart marker header yet
        if self.typ() >= 64 {
            Err(Error::Type)
        } else {
            Ok(())
        }
    }

    pub fn check_qvalue(&self) -> Result<(), Error> {
        // Doesn't handle quantization table header yet
        if self.typ() >= 128 {
            Err(Error::QValue)
        } else {
            Ok(())
        }
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    // TODO - assumes no restart marker header or quantization table header
    pub fn header_len() -> usize {
        field::PAYLOAD.start
    }

    #[inline]
    pub fn type_specific(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::TYPE_SPEC]
    }

    #[inline]
    pub fn fragment_offset(&self) -> u32 {
        let data = self.buffer.as_ref();
        BigEndian::read_u24(&data[field::FRAG_OFFSET])
    }

    #[inline]
    pub fn typ(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::TYPE]
    }

    #[inline]
    pub fn qvalue(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::QVALUE]
    }

    /// Returns width in pixels
    #[inline]
    pub fn width(&self) -> u16 {
        let data = self.buffer.as_ref();
        u16::from(data[field::WIDTH]) * WIDTH_HEIGHT_MULT
    }

    /// Returns height in pixels
    #[inline]
    pub fn height(&self) -> u16 {
        let data = self.buffer.as_ref();
        u16::from(data[field::HEIGHT]) * WIDTH_HEIGHT_MULT
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Header<&'a T> {
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        let data = self.buffer.as_ref();
        &data[field::PAYLOAD]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static PACKET_BYTES: [u8; 36] = [
        0x00, 0x00, 0x05, 0x8C, 0x01, 0x3F, 0x50, 0x3C, 0x2F, 0x6F, 0xCF, 0xB7, 0x6E, 0x72, 0xA0,
        0xF4, 0xC9, 0xF5, 0xAD, 0xAF, 0x07, 0x6A, 0xD7, 0xDF, 0xDA, 0x96, 0x7A, 0x6F, 0x9F, 0xFE,
        0x89, 0xF3, 0xFE, 0xEF, 0x62, 0xFF,
    ];

    static PAYLOAD_BYTES: [u8; 28] = [
        0x2F, 0x6F, 0xCF, 0xB7, 0x6E, 0x72, 0xA0, 0xF4, 0xC9, 0xF5, 0xAD, 0xAF, 0x07, 0x6A, 0xD7,
        0xDF, 0xDA, 0x96, 0x7A, 0x6F, 0x9F, 0xFE, 0x89, 0xF3, 0xFE, 0xEF, 0x62, 0xFF,
    ];

    #[test]
    fn header_len() {
        assert_eq!(Header::<&[u8]>::header_len(), MAIN_SIZE);
    }

    #[test]
    fn deconstruct_first() {
        let h = Header::new_checked(&PACKET_BYTES[..]).unwrap();
        assert_eq!(h.type_specific(), 0);
        assert_eq!(h.fragment_offset(), 1420);
        assert_eq!(h.typ(), 1);
        assert_eq!(h.qvalue(), 63);
        assert_eq!(h.width(), 640);
        assert_eq!(h.height(), 480);
        assert_eq!(h.payload().len(), PAYLOAD_BYTES.len());
        assert_eq!(h.payload(), &PAYLOAD_BYTES[..]);
    }
}
