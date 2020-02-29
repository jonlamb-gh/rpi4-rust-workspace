use crate::cache;
use bcm2711::mbox::{Status, WriteAddr, MBOX};
use core::convert::TryFrom;
use core::mem;
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{asm, barrier};
use static_assertions::{assert_eq_size, const_assert_eq};

mod msg;
mod tag_id;

pub use crate::mailbox::msg::*;
pub use crate::mailbox::tag_id::TagId;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Error {
    UnkownTagId(u32),
    Truncated,
    MessageWordAlign,
    Malformed,
    /// The response buffer has error bit(s) set
    BadRequest,
    /// Status word was not recognized
    BadStatusWord,
    /// Unknown error
    Unknown,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Channel {
    /// Property channel
    Prop = 8,
}

impl From<Channel> for u32 {
    fn from(c: Channel) -> u32 {
        c as u32
    }
}

#[repr(C, align(16))]
pub struct MailboxBuffer(
    // The address for buffer needs to be 16-byte aligned so that the
    // Videcore can handle it properly.
    [u32; BUFFER_LEN],
);

impl MailboxBuffer {
    pub fn new() -> Self {
        MailboxBuffer([0; BUFFER_LEN])
    }

    pub fn as_ptr(&self) -> *const u32 {
        self.0.as_ptr() as *const _
    }

    pub fn as_paddr(&self) -> usize {
        self.as_ptr() as usize
    }
}

assert_eq_size!(MailboxBuffer, [u32; 36]);
const_assert_eq!(BUFFER_LEN, 36);
const_assert_eq!(BUFFER_SIZE, 36 * 4);

/// Mailbox abstraction
pub struct Mailbox {
    mbox: MBOX,
    msg_buffer: MailboxBuffer,
}

impl Mailbox {
    pub fn new(mbox: MBOX) -> Self {
        let msg_buffer = MailboxBuffer::new();
        assert_eq!(msg_buffer.as_paddr() & 0xF, 0);
        Mailbox { mbox, msg_buffer }
    }

    /// Returns a newly allocated high-level representation of the response
    pub fn call<R: MsgEmitter>(&mut self, channel: Channel, req: &R) -> Result<RespMsg> {
        // TODO
        // - add size/etc utils for new_checked() fn's
        // - check resp capacity

        let mut msg = Msg::new_unchecked(&mut self.msg_buffer.0[..]);

        // Emit into our local buffer
        req.emit_msg(&mut msg)?;

        // Insert a compiler fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        compiler_fence(Ordering::Release);

        unsafe {
            cache::clean_data_cache_range(
                self.msg_buffer.as_paddr(),
                mem::size_of::<MailboxBuffer>(),
            )
        };

        // Wait until we can write to the mailbox
        loop {
            if self.mbox.status.is_set(Status::Full::Read) == false {
                break;
            }
            asm::nop();
        }

        // Write the physical address of our message
        // to the mailbox with channel identifier
        let buffer_paddr = self.msg_buffer.as_paddr() as u32;
        self.mbox.write_addr.modify(
            WriteAddr::Addr::Field::new((buffer_paddr & !0xF) | (u32::from(channel) & 0xF))
                .unwrap(),
        );

        // Wait for a response
        loop {
            loop {
                if self.mbox.status.is_set(Status::Empty::Read) == false {
                    break;
                }
                asm::nop();
            }

            let resp_word = self.mbox.read_addr.read();

            // Check if it is a response to our message
            if ((resp_word & 0xF) == channel.into()) && ((resp_word & !0xF) == buffer_paddr) {
                unsafe { barrier::dmb(barrier::SY) };

                unsafe {
                    cache::invalidate_data_cache_range(
                        self.msg_buffer.as_paddr(),
                        mem::size_of::<MailboxBuffer>(),
                    )
                };

                let msg = Msg::new_unchecked(&self.msg_buffer.0[..]);

                return match msg.reqresp_code() {
                    ReqRespCode::ResponseSuccess => Ok(RespMsg::try_from(msg)?),
                    ReqRespCode::ResponseError => Err(Error::BadRequest),
                    _ => Err(Error::BadStatusWord),
                };
            }
        }
    }
}
