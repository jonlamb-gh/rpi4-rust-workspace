use core::convert::TryFrom;
use crate::mailbox::{Error, Result, TagId};

/// tagId, buffer size in bytes, resp size is bytes, 3 words
const HEADER_LEN: usize = 3;
const HEADER_SIZE: usize = HEADER_LEN * 4;

/// Responses should have bit 31 set in response length
const RESPONSE_BIT_MASK: u32 = 0x8000_0000;

#[derive(Debug, PartialEq)]
pub struct Tag<T: AsRef<[u32]>> {
    buffer: T,
}

impl<T: AsRef<[u32]>> Tag<T> {
    pub fn new_unchecked(buffer: T) -> Tag<T> {
        Tag { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<Tag<T>> {
        let req = Self::new_unchecked(buffer);
        req.check_len()?;
        Ok(req)
    }

    pub fn check_len(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < HEADER_LEN {
            Err(Error::Truncated)
        } else {
            Ok(())
        }
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    pub fn header_size() -> usize {
        HEADER_SIZE
    }

    #[inline]
    pub fn tag_id(&self) -> Result<TagId> {
        let data = self.buffer.as_ref();
        TagId::try_from(data[0])
    }

    /// Size in bytes
    #[inline]
    pub fn request_size(&self) -> usize {
        let data = self.buffer.as_ref();
        data[1] as usize
    }

    /// Size in bytes
    #[inline]
    pub fn response_size(&self) -> usize {
        let data = self.buffer.as_ref();
        (data[2] & !RESPONSE_BIT_MASK) as usize
    }
}

impl<'a, T: AsRef<[u32]> + ?Sized> Tag<&'a T> {
    #[inline]
    pub fn payload(&self) -> &'a [u32] {
        let data = self.buffer.as_ref();
        &data[3..]
    }
}

impl<T: AsRef<[u32]> + AsMut<[u32]>> Tag<T> {
    #[inline]
    pub fn set_tag_id(&mut self, tag_id: TagId) {
        let data = self.buffer.as_mut();
        data[0] = tag_id.into();
    }

    /// Size in bytes
    #[inline]
    pub fn set_request_size(&mut self, size: usize) {
        let data = self.buffer.as_mut();
        data[1] = size as _;
    }

    /// Size in bytes
    #[inline]
    pub fn set_response_size(&mut self, size: usize) {
        let data = self.buffer.as_mut();
        data[2] = size as u32 | RESPONSE_BIT_MASK;
    }

    #[inline]
    pub fn payload_mut(&mut self) -> &mut [u32] {
        let data = self.buffer.as_mut();
        &mut data[3..]
    }
}

impl<T: AsRef<[u32]>> AsRef<[u32]> for Tag<T> {
    fn as_ref(&self) -> &[u32] {
        self.buffer.as_ref()
    }
}
