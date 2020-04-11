//! [RFC3984](https://tools.ietf.org/html/rfc3984#section-5.2)
//!
//! NOTE: only supports NAL unit types:
//! * 1 (single NAL unit packed per H.264)
//! * 6 (SEI)
//! * 7 (SPS)
//! * 8 (PPS)
//! * 28 (FU-A fragmentation unit)

use crate::NalUnitType;

const SUPPORTED_UNIT_TYPES: [NalUnitType; 5] = [
    NalUnitType::SingleNalUnit,
    NalUnitType::Sei,
    NalUnitType::Sps,
    NalUnitType::Pps,
    NalUnitType::FuA,
];

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Error {
    Truncated,
    Syntax,
    NalUnitType(NalUnitType),
}

/// NAL unit header
///
/// [RFC2435](https://tools.ietf.org/html/rfc2435#section-1.3)
#[derive(Debug, Clone)]
pub struct Header<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    type Rest = ::core::ops::RangeFrom<usize>;

    /// forbidden_zero_bit (1 bit), nal_ref_idc (2 bits), nal_unit_type (5 bits)
    pub const BYTE0: usize = 0;
    /// Remaining
    pub const PAYLOAD: Rest = 1..;
}

impl<T: AsRef<[u8]>> Header<T> {
    pub fn new_unchecked(buffer: T) -> Header<T> {
        Header { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<Header<T>, Error> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        packet.check_forbidden_zero_bit()?;
        packet.check_nal_unit_type()?;
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

    pub fn check_forbidden_zero_bit(&self) -> Result<(), Error> {
        if self.forbidden_zero_bit() {
            Err(Error::Syntax)
        } else {
            Ok(())
        }
    }

    pub fn check_nal_unit_type(&self) -> Result<(), Error> {
        let typ = self.nal_unit_type();
        if !(&SUPPORTED_UNIT_TYPES).contains(&typ) {
            Err(Error::NalUnitType(typ))
        } else {
            Ok(())
        }
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    pub fn header_len() -> usize {
        field::PAYLOAD.start
    }

    #[inline]
    pub fn forbidden_zero_bit(&self) -> bool {
        let data = self.buffer.as_ref();
        (data[field::BYTE0] >> 7) & 0x01 != 0
    }

    #[inline]
    pub fn nal_ref_idc(&self) -> u8 {
        let data = self.buffer.as_ref();
        (data[field::BYTE0] >> 5) & 0x03
    }

    #[inline]
    pub fn nal_unit_type(&self) -> NalUnitType {
        let data = self.buffer.as_ref();
        NalUnitType::from(data[field::BYTE0] & 0x1F)
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> Header<&'a T> {
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        let data = self.buffer.as_ref();
        &data[field::PAYLOAD]
    }
}
