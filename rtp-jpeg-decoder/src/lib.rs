//! [RFC2435](https://tools.ietf.org/html/rfc2435)
//!
//! Some of the things take from:
//! https://github.com/image-rs/image/blob/master/src/jpeg/encoder.rs

#![no_std]

use crate::header::Header;
use byteorder::{BigEndian, ByteOrder};
pub use rtp;

pub mod header;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Error {
    BadFirstPacket,
    StorageOverflow,
    TableSize,
    RtpPayloadType(u8),
    Header(header::Error),
}

/// JPEG payload type
///
/// [RFC189](https://tools.ietf.org/html/rfc1890)
pub const RTP_PAYLOAD_TYPE_JPEG: u8 = 26;

#[derive(Debug)]
pub struct JPEGDecoder<'b> {
    // nanojpeg..
    /// Waits for the first packet with MARKER bit set, starts
    /// decoding on the following first frame packet
    first_marker_found: bool,
    buffered: usize,
    buffer: &'b mut [u8],
}

impl<'b> JPEGDecoder<'b> {
    pub fn new(defrag_storage: &'b mut [u8]) -> Result<Self, Error> {
        // TODO - update this
        let min_size = LUMA_QTABLE.len() + CHROMA_QTABLE.len();

        if defrag_storage.len() < min_size {
            return Err(Error::StorageOverflow);
        }

        Ok(JPEGDecoder {
            first_marker_found: false,
            buffered: 0,
            buffer: defrag_storage,
        })
    }

    // TODO - result type, pass up nanojpg info on full image decode
    pub fn decode(&mut self, packet: &rtp::Packet<&[u8]>) -> Result<(), Error> {
        let rtp_payload_type = packet.payload_type();
        if rtp_payload_type != RTP_PAYLOAD_TYPE_JPEG {
            return Err(Error::RtpPayloadType(rtp_payload_type));
        }

        if !self.first_marker_found {
            if packet.contains_marker() {
                self.first_marker_found = true;
                self.buffered = 0;
            }
            Ok(())
        } else {
            let hdr = Header::new_checked(packet.payload())?;

            if self.buffered == 0 && hdr.fragment_offset() != 0 {
                // Wait until next frame
                self.first_marker_found = false;
                self.buffered = 0;
                return Err(Error::BadFirstPacket);
            }

            // Generate q tables and headers when first packet is recv'd
            if self.buffered == 0 {
                self.generate_headers(&hdr)?;
            }

            // Buffer the fragment
            let payload_size = hdr.payload().len();
            if (self.buffered + payload_size) > self.buffer.len() {
                return Err(Error::StorageOverflow);
            }
            self.buffer[self.buffered..self.buffered + payload_size].copy_from_slice(hdr.payload());
            self.buffered += payload_size;

            // TODO - at the end, check for EOI Marker, write one if needed
            if packet.contains_marker() {
                // EOI ...

                // Run it through the nanojpeg decoder
                todo!();
            }

            todo!()
        }
    }

    fn generate_headers(&mut self, hdr: &Header<&[u8]>) -> Result<(), Error> {
        // MakeTables
        //
        // MakeHeaders
        //   MakeQuantHeader
        //   MakeHuffmanHeader
        //   MakeDRIHeader
        //
        // the rust lib doest SOI, SOF, then DQT...
        // the rfc does SOI, DQT.., then SOF

        let mut lqt = [0; 64];
        let mut cqt = [0; 64];
        make_tables(hdr.qvalue(), &mut lqt, &mut cqt)?;

        self.write_segment(SOI, None)?;

        // DQT's
        for (i, table) in [lqt, cqt].iter().enumerate() {
            self.generate_quantization_header(i as u8, table)?;
        }

        // SOF
        self.generate_frame_header(8, hdr.width(), hdr.height(), hdr.typ())?;

        // DHT's

        todo!()
    }

    fn generate_quantization_header(&mut self, identifier: u8, qtable: &[u8]) -> Result<(), Error> {
        assert_eq!(qtable.len() % 64, 0);
        self.write_segment(DQT, Some(1 + qtable.len() as u16))?;
        self.write_u8(identifier)?;

        // TODO - stuff from build_quantization_segment()
        //
        //let p = if precision == 8 { 0 } else { 1 };
        //let pqtq = (p << 4) | identifier;
        //self.write_u8(pqtq)?;

        for &i in &UNZIGZAG[..] {
            self.write_u8(qtable[i as usize])?;
        }
        Ok(())
    }

    fn generate_frame_header(
        &mut self,
        precision: u8,
        width: u16,
        height: u16,
        typ: u8,
    ) -> Result<(), Error> {
        self.write_segment(SOF, Some(15))?; // TODO
        self.write_u8(precision)?;
        self.write_u16(height)?;
        self.write_u16(width)?;
        self.write_u8(3)?; // 3 components

        // Component 0
        self.write_u8(0)?;
        if typ == 0 {
            self.write_u8(0x21)?; // hsamp = 2, vsamp = 1
        } else {
            self.write_u8(0x22)?; // hsamp = 2, vsamp = 2
        }
        self.write_u8(0)?; // Quant table 0

        // Component 1
        self.write_u8(1)?;
        self.write_u8(0x11)?; // hsamp = 1, vsamp = 1
        self.write_u8(1)?; // Quant table 1

        // Component 2
        self.write_u8(2)?;
        self.write_u8(0x11)?; // hsamp = 1, vsamp = 1
        self.write_u8(1)?; // Quant table 1

        Ok(())
    }

    fn write_segment(&mut self, marker: u8, data_size: Option<u16>) -> Result<(), Error> {
        self.write_all(&[0xFF, marker])?;
        if let Some(s) = data_size {
            self.write_u16(2 + s)?;
        }
        Ok(())
    }

    fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        let data_size = data.len();
        if (self.buffered + data_size) > self.buffer.len() {
            return Err(Error::StorageOverflow);
        }

        self.buffer[self.buffered..self.buffered + data_size].copy_from_slice(data);
        self.buffered += data_size;

        Ok(())
    }

    fn write_u16(&mut self, data: u16) -> Result<(), Error> {
        let data_size = 2;
        if (self.buffered + data_size) > self.buffer.len() {
            return Err(Error::StorageOverflow);
        }
        BigEndian::write_u16(
            &mut self.buffer[self.buffered..self.buffered + data_size],
            data,
        );
        self.buffered += data_size;
        Ok(())
    }

    fn write_u8(&mut self, data: u8) -> Result<(), Error> {
        let data_size = 1;
        if (self.buffered + data_size) > self.buffer.len() {
            return Err(Error::StorageOverflow);
        }
        self.buffer[self.buffered] = data;
        self.buffered += data_size;
        Ok(())
    }
}

fn make_tables(qvalue: u8, lqt: &mut [u8], cqt: &mut [u8]) -> Result<(), Error> {
    if (lqt.len() != LUMA_QTABLE.len()) || (cqt.len() != CHROMA_QTABLE.len()) {
        return Err(Error::TableSize);
    }

    let scale = u32::from(clamp(qvalue, 1, 100));
    let scale = if scale < 50 {
        5000 / scale
    } else {
        200 - scale * 2
    };

    let scale_value = |&v: &u8| {
        let value = (u32::from(v) * scale + 50) / 100;
        clamp(value, 1, u32::from(u8::max_value())) as u8
    };

    LUMA_QTABLE
        .iter()
        .enumerate()
        .for_each(|(idx, v)| lqt[idx] = scale_value(v));
    CHROMA_QTABLE
        .iter()
        .enumerate()
        .for_each(|(idx, v)| cqt[idx] = scale_value(v));

    Ok(())
}

// Taken from https://github.com/image-rs/image/blob/master/src/math/utils.rs
#[inline]
pub fn clamp<N>(a: N, min: N, max: N) -> N
where
    N: PartialOrd,
{
    if a < min {
        return min;
    }
    if a > max {
        return max;
    }
    a
}

impl From<header::Error> for Error {
    fn from(e: header::Error) -> Error {
        Error::Header(e)
    }
}

/// Table K.1
#[rustfmt::skip]
static LUMA_QTABLE: [u8; 64] = [
    16, 11, 10, 16,  24,  40,  51,  61,
    12, 12, 14, 19,  26,  58,  60,  55,
    14, 13, 16, 24,  40,  57,  69,  56,
    14, 17, 22, 29,  51,  87,  80,  62,
    18, 22, 37, 56,  68, 109, 103,  77,
    24, 35, 55, 64,  81, 104, 113,  92,
    49, 64, 78, 87, 103, 121, 120, 101,
    72, 92, 95, 98, 112, 100, 103,  99,
];

/// Table K.2
#[rustfmt::skip]
static CHROMA_QTABLE: [u8; 64] = [
    17, 18, 24, 47, 99, 99, 99, 99,
    18, 21, 26, 66, 99, 99, 99, 99,
    24, 26, 56, 99, 99, 99, 99, 99,
    47, 66, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
    99, 99, 99, 99, 99, 99, 99, 99,
];

/// The permutation of dct coefficients.
#[rustfmt::skip]
static UNZIGZAG: [u8; 64] = [
     0,  1,  8, 16,  9,  2,  3, 10,
    17, 24, 32, 25, 18, 11,  4,  5,
    12, 19, 26, 33, 40, 48, 41, 34,
    27, 20, 13,  6,  7, 14, 21, 28,
    35, 42, 49, 56, 57, 50, 43, 36,
    29, 22, 15, 23, 30, 37, 44, 51,
    58, 59, 52, 45, 38, 31, 39, 46,
    53, 60, 61, 54, 47, 55, 62, 63,
];

// Markers
// Baseline DCT
static SOF: u8 = 0xC0;
// Huffman Tables
static DHT: u8 = 0xC4;
// Start of Image (standalone)
static SOI: u8 = 0xD8;
// End of image (standalone)
static EOI: u8 = 0xD9;
// Start of Scan
static SOS: u8 = 0xDA;
// Quantization Tables
static DQT: u8 = 0xDB;
// Application segments start and end
static APP0: u8 = 0xE0;
