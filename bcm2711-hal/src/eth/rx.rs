use crate::eth::{Descriptor, LEADING_PAD};
use core::ops::{Deref, DerefMut};

pub struct RxPacket<'a> {
    pub(crate) entry: &'a mut Descriptor,
    pub(crate) length: usize,
}

impl<'a> RxPacket<'a> {
    pub fn len(&self) -> usize {
        self.length - LEADING_PAD
    }
}

impl<'a> Deref for RxPacket<'a> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        // To cater for the IP header alignment the hardware does.
        // This would actually not be needed if we don't program
        // RBUF_ALIGN_2B
        &self.entry.as_slice()[LEADING_PAD..self.length]
    }
}

impl<'a> DerefMut for RxPacket<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // To cater for the IP header alignment the hardware does.
        // This would actually not be needed if we don't program
        // RBUF_ALIGN_2B
        &mut self.entry.as_mut_slice()[LEADING_PAD..self.length]
    }
}
