use bcm2711::mbox::*;
use core::convert::TryFrom;
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{asm, barrier};

mod msg;
mod tag_id;

pub use crate::mailbox::msg::*;
pub use crate::mailbox::tag_id::TagId;

pub type Result<T> = core::result::Result<T, Error>;

// TODO - redo these
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
#[repr(align(16))]
pub struct MailboxBuffer(
    // The address for buffer needs to be 16-byte aligned so that the
    // Videcore can handle it properly.
    [u32; BUFFER_LEN],
);

/// Mailbox abstraction
pub struct Mailbox {
    mbox: MBOX,
    msg_buffer: MailboxBuffer,
}

impl Mailbox {
    pub fn new(mbox: MBOX) -> Self {
        Mailbox {
            mbox,
            msg_buffer: MailboxBuffer([0; BUFFER_LEN]),
        }
    }

    /// Returns a newly allocated high-level representation of the response
    pub fn call<R: MsgEmitter>(&mut self, channel: Channel, req: &R) -> Result<RespMsg> {
        // TODO
        // - add size/etc utils for new_checked() fn's
        // - check resp capacity
        // - check buffer alignment , ie #[repr(align(16))]

        let buffer_paddr = self.msg_buffer.0.as_ptr() as u32;
        let mut msg = Msg::new_unchecked(&mut self.msg_buffer.0[..]);

        // Emit into our local buffer
        req.emit_msg(&mut msg)?;

        // Insert a compiler fence that ensures that all stores to the
        // mbox buffer are finished before the GPU is signaled (which
        // is done by a store operation as well).
        compiler_fence(Ordering::Release);

        // Wait until we can write to the mailbox
        loop {
            if self.mbox.STATUS.is_set(STATUS::FULL) == false {
                break;
            }
            asm::nop();
        }

        // Write the physical address of our message
        // to the mailbox with channel identifier
        self.mbox
            .WRITE
            .set((buffer_paddr & !0xF) | (u32::from(channel) & 0xF));

        // Wait for a response
        loop {
            loop {
                if self.mbox.STATUS.is_set(STATUS::EMPTY) == false {
                    break;
                }
                asm::nop();
            }

            let resp_word = self.mbox.READ.get();

            // Check if it is a response to our message
            if ((resp_word & 0xF) == channel.into()) && ((resp_word & !0xF) == buffer_paddr) {
                unsafe { barrier::dmb(barrier::SY) };

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
