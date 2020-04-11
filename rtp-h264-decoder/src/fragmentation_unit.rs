#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Error {
    Truncated,
}

/// NAL fragmentation unit
///
/// [RFC2435](https://tools.ietf.org/html/rfc2435#section-5.8)
#[derive(Debug, Clone)]
pub struct FragmentationUnit<T: AsRef<[u8]>> {
    buffer: T,
}

mod field {
    type Rest = ::core::ops::RangeFrom<usize>;

    /// FU header byte
    /// Start (1 bit), end (1 bit), reserved (1 bit),
    /// NAL unit payload type (5 bits)
    pub const HEADER_BYTE: usize = 0;
    /// Remaining
    pub const PAYLOAD: Rest = 1..;
}

impl<T: AsRef<[u8]>> FragmentationUnit<T> {
    pub fn new_unchecked(buffer: T) -> FragmentationUnit<T> {
        FragmentationUnit { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<FragmentationUnit<T>, Error> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
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

    pub fn into_inner(self) -> T {
        self.buffer
    }

    pub fn header_len() -> usize {
        field::PAYLOAD.start
    }

    #[inline]
    pub fn start(&self) -> bool {
        let data = self.buffer.as_ref();
        (data[field::HEADER_BYTE] >> 7) & 0x01 != 0
    }

    #[inline]
    pub fn end(&self) -> bool {
        let data = self.buffer.as_ref();
        (data[field::HEADER_BYTE] >> 6) & 0x01 != 0
    }

    #[inline]
    pub fn typ(&self) -> u8 {
        let data = self.buffer.as_ref();
        data[field::HEADER_BYTE] & 0x1F
    }
}

impl<'a, T: AsRef<[u8]> + ?Sized> FragmentationUnit<&'a T> {
    #[inline]
    pub fn payload(&self) -> &'a [u8] {
        let data = self.buffer.as_ref();
        &data[field::PAYLOAD]
    }
}
