//! VideoCore Mailbox messages and utilities

mod alloc_framebuffer;
mod get_arm_mem;
mod get_clock_rate;
mod get_serial_num;
mod get_temp;
mod get_vc_mem;
mod tag;

use crate::mailbox::{Error, Result, TagId};
use core::convert::TryFrom;
use core::fmt;

pub use self::alloc_framebuffer::{
    PixelOrder, Repr as AllocFramebufferRepr, Req as AllocFramebufferReq,
    Resp as AllocFramebufferResp,
};
pub use self::get_arm_mem::{Repr as GetArmMemRepr, Req as GetArmMemReq, Resp as GetArmMemResp};
pub use self::get_clock_rate::{
    Repr as GetClockRateRepr, Req as GetClockRateReq, Resp as GetClockRateResp,
};
pub use self::get_serial_num::{
    Repr as GetSerialNumRepr, Req as GetSerialNumReq, Resp as GetSerialNumResp,
};
pub use self::get_temp::{Repr as GetTempRepr, Req as GetTempReq, Resp as GetTempResp};
pub use self::get_vc_mem::{Repr as GetVcMemRepr, Req as GetVcMemReq, Resp as GetVcMemResp};
pub use self::tag::Tag;

pub const BUFFER_LEN: usize = 36;
pub const BUFFER_SIZE: usize = BUFFER_LEN * 4;
pub const LAST_TAG_SIZE: usize = 4;

/// First word is total size, second word is req/resp code
const HEADER_LEN: usize = 2;
const HEADER_SIZE: usize = HEADER_LEN * 4;

// TODO - make enum combined with response_status
pub const REQUEST_CODE: u32 = 0;
pub const RESPONSE_SUCCESS_CODE: u32 = 0x8000_0000;
pub const RESPONSE_ERROR_CODE: u32 = 0x8000_0001;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ReqRespCode {
    Request,
    ResponseSuccess,
    ResponseError,
    Unknown(u32),
}

pub trait MsgEmitter {
    /// Return the size of a message that will be emitted from this high-level
    /// representation
    fn msg_size(&self) -> usize;

    fn emit_msg<T: AsRef<[u32]> + AsMut<[u32]>>(&self, msg: &mut Msg<T>) -> Result<()>;
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ReqMsg {
    GetTemp(GetTempRepr),
    GetArmMem(GetArmMemRepr),
    GetVcMem(GetVcMemRepr),
    GetClockRate(GetClockRateRepr),
    GetSerialNum(GetSerialNumRepr),
    AllocFramebuffer(AllocFramebufferRepr),
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum RespMsg {
    GetTemp(GetTempRepr),
    GetArmMem(GetArmMemRepr),
    GetVcMem(GetVcMemRepr),
    GetClockRate(GetClockRateRepr),
    GetSerialNum(GetSerialNumRepr),
    AllocFramebuffer(AllocFramebufferRepr),
}

#[derive(Debug)]
pub struct Msg<T: AsRef<[u32]>> {
    buffer: T,
}

impl<T: AsRef<[u32]>> Msg<T> {
    pub fn new_unchecked(buffer: T) -> Msg<T> {
        Msg { buffer }
    }

    pub fn new_checked(buffer: T) -> Result<Msg<T>> {
        let packet = Self::new_unchecked(buffer);
        packet.check_len()?;
        Ok(packet)
    }

    pub fn check_len(&self) -> Result<()> {
        let len = self.buffer.as_ref().len();
        if len < BUFFER_LEN {
            Err(Error::Truncated)
        } else if (len * 4) < self.buffer_size() {
            Err(Error::Malformed)
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

    /// Size in bytes
    #[inline]
    pub fn buffer_size(&self) -> usize {
        let data = self.buffer.as_ref();
        data[0] as usize
    }

    #[inline]
    pub fn reqresp_code(&self) -> ReqRespCode {
        let data = self.buffer.as_ref();
        let code = data[1];
        match code {
            REQUEST_CODE => ReqRespCode::Request,
            RESPONSE_SUCCESS_CODE => ReqRespCode::ResponseSuccess,
            RESPONSE_ERROR_CODE => ReqRespCode::ResponseError,
            _ => ReqRespCode::Unknown(code),
        }
    }
}

impl<'a, T: AsRef<[u32]> + ?Sized> Msg<&'a T> {
    #[inline]
    pub fn payload(&self) -> &'a [u32] {
        let data = self.buffer.as_ref();
        &data[2..]
    }
}

impl<T: AsRef<[u32]> + AsMut<[u32]>> Msg<T> {
    #[inline]
    pub fn set_buffer_size(&mut self, size: usize) {
        let data = self.buffer.as_mut();
        data[0] = size as u32;
    }

    #[inline]
    pub fn set_reqresp_code(&mut self, code: ReqRespCode) {
        let data = self.buffer.as_mut();
        data[1] = match code {
            ReqRespCode::Request => REQUEST_CODE,
            ReqRespCode::ResponseSuccess => RESPONSE_SUCCESS_CODE,
            ReqRespCode::ResponseError => RESPONSE_ERROR_CODE,
            ReqRespCode::Unknown(c) => c,
        };
    }

    #[inline]
    pub fn payload_mut(&mut self) -> &mut [u32] {
        let data = self.buffer.as_mut();
        &mut data[2..]
    }

    #[inline]
    pub fn fill_last_tag(&mut self) -> Result<()> {
        let data = self.buffer.as_mut();
        let size_bytes = data[0];

        if (size_bytes % 4) != 0 {
            Err(Error::MessageWordAlign)
        } else {
            let size_words = size_bytes as usize / 4;
            data[size_words - 1] = TagId::Last.into();
            Ok(())
        }
    }
}

impl<T: AsRef<[u32]>> AsRef<[u32]> for Msg<T> {
    fn as_ref(&self) -> &[u32] {
        self.buffer.as_ref()
    }
}

impl<'a, T: AsRef<[u32]> + ?Sized> TryFrom<Msg<&'a T>> for RespMsg {
    type Error = Error;

    fn try_from(m: Msg<&'a T>) -> Result<RespMsg> {
        let tag = Tag::new_checked(m.payload())?;
        let tag_id = tag.tag_id()?;

        match tag_id {
            TagId::GetTemperature => Ok(RespMsg::GetTemp(GetTempRepr::parse_response(&m)?)),
            TagId::GetArmMem => Ok(RespMsg::GetArmMem(GetArmMemRepr::parse_response(&m)?)),
            TagId::GetVcMem => Ok(RespMsg::GetVcMem(GetVcMemRepr::parse_response(&m)?)),
            TagId::GetClockRate => Ok(RespMsg::GetClockRate(GetClockRateRepr::parse_response(&m)?)),
            TagId::GetSerialNum => Ok(RespMsg::GetSerialNum(GetSerialNumRepr::parse_response(&m)?)),
            // TODO - frame buffer currently just matches on the first TagId
            TagId::SetPhySize => Ok(RespMsg::AllocFramebuffer(
                AllocFramebufferRepr::parse_response(&m)?,
            )),
            _ => Err(Error::UnkownTagId(tag_id.into())),
        }
    }
}

impl<T: AsRef<[u32]>> fmt::Display for Msg<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Mbox Msg {{ Size {} Code {:?} }}",
            self.buffer_size(),
            self.reqresp_code()
        )
    }
}

impl MsgEmitter for &ReqMsg {
    fn msg_size(&self) -> usize {
        match *self {
            ReqMsg::GetTemp(repr) => repr.msg_size(),
            ReqMsg::GetArmMem(repr) => repr.msg_size(),
            ReqMsg::GetVcMem(repr) => repr.msg_size(),
            ReqMsg::GetClockRate(repr) => repr.msg_size(),
            ReqMsg::GetSerialNum(repr) => repr.msg_size(),
            ReqMsg::AllocFramebuffer(repr) => repr.msg_size(),
        }
    }

    fn emit_msg<T: AsRef<[u32]> + AsMut<[u32]>>(&self, msg: &mut Msg<T>) -> Result<()> {
        match *self {
            ReqMsg::GetTemp(repr) => repr.emit_msg(msg),
            ReqMsg::GetArmMem(repr) => repr.emit_msg(msg),
            ReqMsg::GetVcMem(repr) => repr.emit_msg(msg),
            ReqMsg::GetSerialNum(repr) => repr.emit_msg(msg),
            ReqMsg::GetClockRate(repr) => repr.emit_msg(msg),
            ReqMsg::AllocFramebuffer(repr) => repr.emit_msg(msg),
        }
    }
}
