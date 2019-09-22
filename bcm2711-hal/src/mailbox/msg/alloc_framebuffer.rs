// The framebuffer command has an array of Tag payloads

use crate::cache::{bus_address_bits, cpu_address_bits};
use crate::mailbox::{Error, Msg, MsgEmitter, ReqRespCode, Result, Tag, TagId, LAST_TAG_SIZE};

const REQ_LEN: usize = 29;
const REQ_SIZE: usize = REQ_LEN * 4;

const RESP_LEN: usize = 29;
const RESP_SIZE: usize = RESP_LEN * 4;

const DEFAULT_ALIGN: usize = 4096;

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

    #[inline]
    pub fn phy_width(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[0]
    }

    #[inline]
    pub fn phy_height(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[1]
    }

    #[inline]
    pub fn virt_width(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[5]
    }

    #[inline]
    pub fn virt_height(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[6]
    }

    #[inline]
    pub fn x_offset(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[10]
    }

    #[inline]
    pub fn y_offset(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[11]
    }

    #[inline]
    pub fn depth(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[15]
    }

    #[inline]
    pub fn pixel_order(&self) -> PixelOrder {
        let data = self.buffer.as_ref();
        data[19].into()
    }

    #[inline]
    pub fn buffer_alignment(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[23]
    }

    #[inline]
    pub fn pitch(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[28]
    }
}

impl<T: AsRef<[u32]> + AsMut<[u32]>> Req<T> {
    #[inline]
    pub fn set_phy_size(&mut self, width: u32, height: u32) {
        let data = self.buffer.as_mut();
        data[0] = width;
        data[1] = height;
    }

    #[inline]
    pub fn set_virt_size(&mut self, width: u32, height: u32) {
        let data = self.buffer.as_mut();
        data[2] = TagId::SetVirtSize.into();
        data[3] = 8;
        data[4] = 8;
        data[5] = width;
        data[6] = height;
    }

    #[inline]
    pub fn set_virt_offset(&mut self, x: u32, y: u32) {
        let data = self.buffer.as_mut();
        data[7] = TagId::SetVirtOffset.into();
        data[8] = 8;
        data[9] = 8;
        data[10] = x;
        data[11] = y;
    }

    #[inline]
    pub fn set_depth(&mut self, depth: u32) {
        let data = self.buffer.as_mut();
        data[12] = TagId::SetDepth.into();
        data[13] = 4;
        data[14] = 4;
        data[15] = depth;
    }

    #[inline]
    pub fn set_pixel_order(&mut self, po: PixelOrder) {
        let data = self.buffer.as_mut();
        data[16] = TagId::SetPixelOrder.into();
        data[17] = 4;
        data[18] = 4;
        data[19] = po.into();
    }

    #[inline]
    pub fn set_alloc_buffer(&mut self, align: u32) {
        let data = self.buffer.as_mut();
        data[20] = TagId::AllocBuffer.into();
        data[21] = 8;
        data[22] = 8;
        data[23] = align;
        data[24] = 0;
    }

    #[inline]
    pub fn set_get_pitch(&mut self) {
        let data = self.buffer.as_mut();
        data[25] = TagId::GetPitch.into();
        data[26] = 4;
        data[27] = 4;
        data[28] = 0;
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
    pub fn phy_width(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[0]
    }

    #[inline]
    pub fn phy_height(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[1]
    }

    #[inline]
    pub fn virt_width(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[5]
    }

    #[inline]
    pub fn virt_height(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[6]
    }

    #[inline]
    pub fn x_offset(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[10]
    }

    #[inline]
    pub fn y_offset(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[11]
    }

    #[inline]
    pub fn depth(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[15]
    }

    #[inline]
    pub fn pixel_order(&self) -> PixelOrder {
        let data = self.buffer.as_ref();
        data[19].into()
    }

    #[inline]
    pub fn buffer_address(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[23]
    }

    #[inline]
    pub fn buffer_size(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[24]
    }

    #[inline]
    pub fn pitch(&self) -> u32 {
        let data = self.buffer.as_ref();
        data[28]
    }
}

impl<T: AsRef<[u32]>> AsRef<[u32]> for Resp<T> {
    fn as_ref(&self) -> &[u32] {
        self.buffer.as_ref()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PixelOrder {
    BGR,
    RGB,
    Unknown(u32),
}

impl From<PixelOrder> for u32 {
    fn from(po: PixelOrder) -> u32 {
        match po {
            PixelOrder::BGR => 0,
            PixelOrder::RGB => 1,
            PixelOrder::Unknown(v) => v,
        }
    }
}

impl From<u32> for PixelOrder {
    fn from(val: u32) -> PixelOrder {
        match val {
            0 => PixelOrder::BGR,
            1 => PixelOrder::RGB,
            _ => PixelOrder::Unknown(val),
        }
    }
}

// TODO - update fields
/// A high-level representation of a AllocFramebuffer command/response
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Repr {
    pub phy_width: usize,
    pub phy_height: usize,
    pub virt_width: usize,
    pub virt_height: usize,
    pub x_offset: usize,
    pub y_offset: usize,
    pub depth: usize,
    pub pixel_order: PixelOrder,
    pub buffer_align: usize,
    buffer_address: u32,
    buffer_size: u32,
    pitch: u32,
}

/// A default Framebuffer request
impl Default for Repr {
    fn default() -> Repr {
        Repr {
            phy_width: 800,
            phy_height: 480,
            virt_width: 800,
            virt_height: 480,
            x_offset: 0,
            y_offset: 0,
            depth: 32,
            pixel_order: PixelOrder::RGB,
            buffer_align: DEFAULT_ALIGN,
            buffer_address: 0,
            buffer_size: 0,
            pitch: 0,
        }
    }
}

impl Repr {
    // TODO - update fields
    pub fn new() -> Self {
        Self::default()
    }

    pub fn alloc_buffer_address(&self) -> u32 {
        self.buffer_address & cpu_address_bits::MASK
    }

    pub fn alloc_buffer_bus_address(&self) -> u32 {
        self.buffer_address | bus_address_bits::ALIAS_4_L2_COHERENT
    }

    pub fn alloc_buffer_size(&self) -> usize {
        self.buffer_size as usize
    }

    pub fn pitch(&self) -> usize {
        self.pitch as usize
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

        struct Item {
            offset: usize,
            tag_id: TagId,
        }

        let items = [
            Item {
                offset: 0,
                tag_id: TagId::SetPhySize,
            },
            Item {
                offset: 5,
                tag_id: TagId::SetVirtSize,
            },
            Item {
                offset: 10,
                tag_id: TagId::SetVirtOffset,
            },
            Item {
                offset: 15,
                tag_id: TagId::SetDepth,
            },
            Item {
                offset: 19,
                tag_id: TagId::SetPixelOrder,
            },
            Item {
                offset: 23,
                tag_id: TagId::AllocBuffer,
            },
            Item {
                offset: 28,
                tag_id: TagId::GetPitch,
            },
        ];

        // Check for the expected TagId's at their offsets
        for item in &items {
            let tag = Tag::new_checked(&(msg.payload()[item.offset..]))?;

            if tag.tag_id()? != item.tag_id {
                return Err(Error::Malformed);
            }
        }

        let tag = Tag::new_checked(msg.payload())?;

        let resp = Resp::new_checked(tag.payload())?;

        Ok(Repr {
            phy_width: resp.phy_width() as _,
            phy_height: resp.phy_height() as _,
            virt_width: resp.virt_width() as _,
            virt_height: resp.virt_height() as _,
            x_offset: resp.x_offset() as _,
            y_offset: resp.y_offset() as _,
            depth: resp.depth() as _,
            pixel_order: resp.pixel_order(),
            buffer_align: DEFAULT_ALIGN,
            buffer_address: resp.buffer_address(),
            buffer_size: resp.buffer_size(),
            pitch: resp.pitch(),
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

        // Setup the first tag here, its payload is the other tags
        tag.set_tag_id(TagId::SetPhySize);
        tag.set_request_size(8);
        tag.set_response_size(8);
        tag.check_len()?;

        let mut req = Req::new_unchecked(tag.payload_mut());

        req.set_phy_size(self.phy_width as _, self.phy_height as _);
        req.set_virt_size(self.virt_width as _, self.virt_height as _);
        req.set_virt_offset(self.x_offset as _, self.y_offset as _);
        req.set_depth(self.depth as _);
        req.set_pixel_order(self.pixel_order);
        req.set_alloc_buffer(DEFAULT_ALIGN as _);
        req.set_get_pitch();
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
