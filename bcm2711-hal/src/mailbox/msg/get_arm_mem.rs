use crate::mailbox::{Error, Msg, MsgEmitter, ReqRespCode, Result, Tag, TagId, LAST_TAG_SIZE};

const TAG: TagId = TagId::GetArmMem;

const REQ_LEN: usize = 2;
const REQ_SIZE: usize = REQ_LEN * 4;

const RESP_LEN: usize = 2;
const RESP_SIZE: usize = RESP_LEN * 4;

#[derive(Debug, PartialEq)]
pub struct Req<T: AsRef<[u32]>> {
    buffer: T,
}

impl<T: AsRef<[u32]>> Req<T> {
    pub fn new_unchecked(buffer: T) -> Req<T> {
        Req { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<Req<T>> {
        let req = Self::new_unchecked(buffer);
        req.check_len()?;
        Ok(req)
    }

    pub fn check_len(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < REQ_LEN {
            Err(Error::Truncated)
        } else {
            Ok(())
        }
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }
}

impl<T: AsRef<[u32]> + AsMut<[u32]>> Req<T> {
    #[inline]
    pub fn zero(&mut self) {
        let data = self.buffer.as_mut();
        data[0] = 0;
        data[1] = 0;
    }
}

impl<T: AsRef<[u32]>> AsRef<[u32]> for Req<T> {
    fn as_ref(&self) -> &[u32] {
        self.buffer.as_ref()
    }
}

#[derive(Debug, PartialEq)]
pub struct Resp<T: AsRef<[u32]>> {
    buffer: T,
}

impl<T: AsRef<[u32]>> Resp<T> {
    pub fn new_unchecked(buffer: T) -> Resp<T> {
        Resp { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<Resp<T>> {
        let req = Self::new_unchecked(buffer);
        req.check_len()?;
        Ok(req)
    }

    pub fn check_len(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < RESP_LEN {
            Err(Error::Truncated)
        } else {
            Ok(())
        }
    }

    pub fn into_inner(self) -> T {
        self.buffer
    }

    #[inline]
    pub fn address(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[0]
    }

    #[inline]
    pub fn size(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[1]
    }
}

impl<T: AsRef<[u32]>> AsRef<[u32]> for Resp<T> {
    fn as_ref(&self) -> &[u32] {
        self.buffer.as_ref()
    }
}

/// A high-level representation of a GetArmMem command/response
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Repr {
    address: u32,
    size: u32,
}

/// A default GetArmMem request
impl Default for Repr {
    fn default() -> Repr {
        Repr {
            address: 0,
            size: 0,
        }
    }
}

impl Repr {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn address(&self) -> u32 {
        self.address
    }

    pub fn size(&self) -> usize {
        self.size as usize
    }

    pub fn parse_response<T: AsRef<[u32]> + ?Sized>(msg: &Msg<&T>) -> Result<Repr> {
        if msg.buffer_size()
            != (Msg::<&T>::header_size() + Tag::<&T>::header_size() + RESP_SIZE + LAST_TAG_SIZE)
        {
            return Err(Error::Malformed);
        }

        if msg.reqresp_code() != ReqRespCode::ResponseSuccess {
            return Err(Error::Malformed);
        }

        let tag = Tag::new_checked(msg.payload())?;

        if tag.tag_id()? != TAG {
            return Err(Error::Malformed);
        }

        if tag.response_size() != RESP_SIZE {
            return Err(Error::Malformed);
        }

        let resp = Resp::new_checked(tag.payload())?;

        Ok(Repr {
            address: resp.address(),
            size: resp.size(),
        })
    }

    /// Return the size of a packet that will be emitted from this high-level
    /// representation
    pub fn buffer_size(&self) -> usize {
        // Request and response are the same size/shape
        RESP_SIZE
    }

    pub fn emit_request<T: AsRef<[u32]> + AsMut<[u32]>>(&self, msg: &mut Msg<T>) -> Result<()> {
        msg.set_buffer_size(
            Msg::<&T>::header_size() + Tag::<&T>::header_size() + REQ_SIZE + LAST_TAG_SIZE,
        );
        msg.set_reqresp_code(ReqRespCode::Request);

        let mut tag = Tag::new_unchecked(msg.payload_mut());

        tag.set_tag_id(TAG);
        tag.set_request_size(REQ_SIZE);
        tag.set_response_size(RESP_SIZE);
        tag.check_len()?;

        let mut req = Req::new_unchecked(tag.payload_mut());

        req.zero();
        req.check_len()?;

        msg.fill_last_tag()?;
        msg.check_len()?;

        Ok(())
    }
}

impl MsgEmitter for Repr {
    fn msg_size(&self) -> usize {
        Msg::<&dyn AsRef<[u32]>>::header_size()
            + Tag::<&dyn AsRef<[u32]>>::header_size()
            + RESP_SIZE
            + LAST_TAG_SIZE
    }

    fn emit_msg<T: AsRef<[u32]> + AsMut<[u32]>>(&self, msg: &mut Msg<T>) -> Result<()> {
        self.emit_request(msg)
    }
}
