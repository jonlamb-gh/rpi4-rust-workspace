//! WIP - just de-fragments for now

#![no_std]

pub use crate::fragmentation_unit::FragmentationUnit;
pub use crate::header::Header;
pub use crate::nal_unit_type::NalUnitType;
use log::trace;
pub use rtp;

pub mod fragmentation_unit;
pub mod header;
pub mod nal_unit_type;

pub const START_SEQ: [u8; 4] = [0x00, 0x00, 0x00, 0x01];

// TODO
pub const RTP_PAYLOAD_TYPE_H264: u8 = 96;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Error {
    Truncated,
    Syntax,
    NalUnitType(NalUnitType),
    StorageOverflow,
    RtpPayloadType(u8),
}

// TODO - wait for SPS/PPS/etc
#[derive(Debug)]
pub struct H264Decoder<'b> {
    last_seq_num: u16,
    buffered: usize,
    buffer: &'b mut [u8],
}

impl<'b> H264Decoder<'b> {
    pub fn new(defrag_storage: &'b mut [u8]) -> Result<Self, Error> {
        // TODO
        let min_size = START_SEQ.len();
        if defrag_storage.len() < min_size {
            return Err(Error::StorageOverflow);
        }

        Ok(H264Decoder {
            last_seq_num: 0,
            buffered: 0,
            buffer: defrag_storage,
        })
    }

    pub fn reset(&mut self) {
        //self.first_marker_found = false;
        self.buffered = 0;
        self.last_seq_num = 0;
    }

    pub fn decode(&mut self, packet: &rtp::Packet<&[u8]>) -> Result<(), Error> {
        // TODO - check len and version on packet
        // - check extension, not supported

        let rtp_payload_type = packet.payload_type();
        if rtp_payload_type != RTP_PAYLOAD_TYPE_H264 {
            return Err(Error::RtpPayloadType(rtp_payload_type));
        }

        trace!("{}", packet);
        trace!("prev sequence_number: {}", self.last_seq_num);

        let hdr = Header::new_checked(packet.payload())?;

        trace!("{}", hdr);

        todo!()
    }
}
