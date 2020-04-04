//! [RFC2435](https://tools.ietf.org/html/rfc2435)
//!
//! Some of the things take from:
//! https://github.com/image-rs/image/blob/master/src/jpeg/encoder.rs

#![no_std]

use crate::header::Header;
use byteorder::{BigEndian, ByteOrder};
pub use nanojpeg_rs::{ImageInfo, NanoJPeg};
pub use rtp;

pub mod header;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Error {
    BadFirstPacket,
    StorageOverflow,
    TableSize,
    RtpPayloadType(u8),
    Header(header::Error),
    Decoder(nanojpeg_rs::Error),
}

/// JPEG payload type
///
/// [RFC189](https://tools.ietf.org/html/rfc1890)
pub const RTP_PAYLOAD_TYPE_JPEG: u8 = 26;

#[derive(Debug)]
pub struct JPEGDecoder<'b> {
    dec: NanoJPeg,
    /// Waits for the first packet with MARKER bit set, starts
    /// decoding on the following first frame packet
    first_marker_found: bool,
    buffered: usize,
    buffer: &'b mut [u8],
}

impl<'b> JPEGDecoder<'b> {
    pub fn new(dec: NanoJPeg, defrag_storage: &'b mut [u8]) -> Result<Self, Error> {
        // TODO - update this
        let min_size = 1024;
        if defrag_storage.len() < min_size {
            return Err(Error::StorageOverflow);
        }

        Ok(JPEGDecoder {
            dec,
            first_marker_found: false,
            buffered: 0,
            buffer: defrag_storage,
        })
    }

    pub fn decode(&mut self, packet: &rtp::Packet<&[u8]>) -> Result<Option<ImageInfo>, Error> {
        // TODO - check len and version on packet

        let rtp_payload_type = packet.payload_type();
        if rtp_payload_type != RTP_PAYLOAD_TYPE_JPEG {
            return Err(Error::RtpPayloadType(rtp_payload_type));
        }

        if !self.first_marker_found {
            if packet.contains_marker() {
                self.first_marker_found = true;
                self.buffered = 0;
                //println!("Found marker, starting to buffer");
            }
            Ok(None)
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
                //println!("Generated headers, buffered {}", self.buffered);
            }

            // Buffer the fragment
            let payload_size = hdr.payload().len();
            if (self.buffered + payload_size) > self.buffer.len() {
                return Err(Error::StorageOverflow);
            }
            self.buffer[self.buffered..self.buffered + payload_size].copy_from_slice(hdr.payload());
            self.buffered += payload_size;

            //println!("payload_size {}", payload_size);
            //println!("fragment_offset {}", hdr.fragment_offset());
            //println!("buffered {}", self.buffered);

            if packet.contains_marker() {
                //println!("Found marker, fragment_offset {}", hdr.fragment_offset());
                //println!("end-1 = 0x{:X}", self.buffer[self.buffered - 2]);
                //println!("end = 0x{:X}", self.buffer[self.buffered - 1]);

                // EOI
                self.write_u8(0xFF)?;
                self.write_segment(EOI, None)?;

                // Reset
                let buffer_size = self.buffered;
                self.buffered = 0;

                // Run it through the nanojpeg decoder
                let info = self.dec.decode(&self.buffer[..buffer_size])?;

                return Ok(Some(info));
            }

            Ok(None)
        }
    }

    fn generate_headers(&mut self, hdr: &Header<&[u8]>) -> Result<(), Error> {
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
        self.generate_huffman_header(
            DCCLASS,
            LUMADESTINATION,
            &STD_LUMA_DC_CODE_LENGTHS,
            &STD_LUMA_DC_VALUES,
        )?;
        self.generate_huffman_header(
            ACCLASS,
            LUMADESTINATION,
            &STD_LUMA_AC_CODE_LENGTHS,
            &STD_LUMA_AC_VALUES,
        )?;
        self.generate_huffman_header(
            DCCLASS,
            CHROMADESTINATION,
            &STD_CHROMA_DC_CODE_LENGTHS,
            &STD_CHROMA_DC_VALUES,
        )?;
        self.generate_huffman_header(
            ACCLASS,
            CHROMADESTINATION,
            &STD_CHROMA_AC_CODE_LENGTHS,
            &STD_CHROMA_AC_VALUES,
        )?;

        // SOS
        self.generate_scan_header()?;
        Ok(())
    }

    fn generate_quantization_header(&mut self, identifier: u8, qtable: &[u8]) -> Result<(), Error> {
        assert_eq!(qtable.len() % 64, 0);
        self.write_segment(DQT, Some(1 + qtable.len() as u16))?;
        self.write_u8(identifier)?;
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
        self.write_segment(SOF, Some(15))?;
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

    fn generate_huffman_header(
        &mut self,
        class: u8,
        destination: u8,
        numcodes: &[u8],
        values: &[u8],
    ) -> Result<(), Error> {
        self.write_segment(DHT, Some(1 + numcodes.len() as u16 + values.len() as u16))?;
        let tcth = (class << 4) | destination;
        self.write_u8(tcth)?;
        assert_eq!(numcodes.len(), 16);
        self.write_all(numcodes)?;
        let mut sum = 0usize;
        for &i in numcodes.iter() {
            sum += i as usize;
        }
        assert_eq!(sum, values.len());
        self.write_all(values)?;
        Ok(())
    }

    fn generate_scan_header(&mut self) -> Result<(), Error> {
        self.write_segment(SOS, Some(10))?;
        self.write_u8(3)?; // 3 components
        self.write_u8(0)?; // Component 0
        self.write_u8(0)?; // Huffman table 0
        self.write_u8(1)?; // Component 1
        self.write_u8(0x11)?; // Huffman table 1
        self.write_u8(2)?; // Component 2
        self.write_u8(0x11)?; // Huffman table 1
        self.write_u8(0)?;
        self.write_u8(63)?;
        self.write_u8(0)?;
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

impl From<nanojpeg_rs::Error> for Error {
    fn from(e: nanojpeg_rs::Error) -> Error {
        Error::Decoder(e)
    }
}

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

static DCCLASS: u8 = 0;
static ACCLASS: u8 = 1;

static LUMADESTINATION: u8 = 0;
static CHROMADESTINATION: u8 = 1;

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

// section K.3
// Code lengths and values for table K.3
static STD_LUMA_DC_CODE_LENGTHS: [u8; 16] = [
    0x00, 0x01, 0x05, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

static STD_LUMA_DC_VALUES: [u8; 12] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
];

// Code lengths and values for table K.4
static STD_CHROMA_DC_CODE_LENGTHS: [u8; 16] = [
    0x00, 0x03, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00,
];

static STD_CHROMA_DC_VALUES: [u8; 12] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0A, 0x0B,
];

// Code lengths and values for table k.5
static STD_LUMA_AC_CODE_LENGTHS: [u8; 16] = [
    0x00, 0x02, 0x01, 0x03, 0x03, 0x02, 0x04, 0x03, 0x05, 0x05, 0x04, 0x04, 0x00, 0x00, 0x01, 0x7D,
];

static STD_LUMA_AC_VALUES: [u8; 162] = [
    0x01, 0x02, 0x03, 0x00, 0x04, 0x11, 0x05, 0x12, 0x21, 0x31, 0x41, 0x06, 0x13, 0x51, 0x61, 0x07,
    0x22, 0x71, 0x14, 0x32, 0x81, 0x91, 0xA1, 0x08, 0x23, 0x42, 0xB1, 0xC1, 0x15, 0x52, 0xD1, 0xF0,
    0x24, 0x33, 0x62, 0x72, 0x82, 0x09, 0x0A, 0x16, 0x17, 0x18, 0x19, 0x1A, 0x25, 0x26, 0x27, 0x28,
    0x29, 0x2A, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49,
    0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69,
    0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x83, 0x84, 0x85, 0x86, 0x87, 0x88, 0x89,
    0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5, 0xA6, 0xA7,
    0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3, 0xC4, 0xC5,
    0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA, 0xE1, 0xE2,
    0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF1, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
    0xF9, 0xFA,
];

// Code lengths and values for table k.6
static STD_CHROMA_AC_CODE_LENGTHS: [u8; 16] = [
    0x00, 0x02, 0x01, 0x02, 0x04, 0x04, 0x03, 0x04, 0x07, 0x05, 0x04, 0x04, 0x00, 0x01, 0x02, 0x77,
];
static STD_CHROMA_AC_VALUES: [u8; 162] = [
    0x00, 0x01, 0x02, 0x03, 0x11, 0x04, 0x05, 0x21, 0x31, 0x06, 0x12, 0x41, 0x51, 0x07, 0x61, 0x71,
    0x13, 0x22, 0x32, 0x81, 0x08, 0x14, 0x42, 0x91, 0xA1, 0xB1, 0xC1, 0x09, 0x23, 0x33, 0x52, 0xF0,
    0x15, 0x62, 0x72, 0xD1, 0x0A, 0x16, 0x24, 0x34, 0xE1, 0x25, 0xF1, 0x17, 0x18, 0x19, 0x1A, 0x26,
    0x27, 0x28, 0x29, 0x2A, 0x35, 0x36, 0x37, 0x38, 0x39, 0x3A, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48,
    0x49, 0x4A, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5A, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68,
    0x69, 0x6A, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7A, 0x82, 0x83, 0x84, 0x85, 0x86, 0x87,
    0x88, 0x89, 0x8A, 0x92, 0x93, 0x94, 0x95, 0x96, 0x97, 0x98, 0x99, 0x9A, 0xA2, 0xA3, 0xA4, 0xA5,
    0xA6, 0xA7, 0xA8, 0xA9, 0xAA, 0xB2, 0xB3, 0xB4, 0xB5, 0xB6, 0xB7, 0xB8, 0xB9, 0xBA, 0xC2, 0xC3,
    0xC4, 0xC5, 0xC6, 0xC7, 0xC8, 0xC9, 0xCA, 0xD2, 0xD3, 0xD4, 0xD5, 0xD6, 0xD7, 0xD8, 0xD9, 0xDA,
    0xE2, 0xE3, 0xE4, 0xE5, 0xE6, 0xE7, 0xE8, 0xE9, 0xEA, 0xF2, 0xF3, 0xF4, 0xF5, 0xF6, 0xF7, 0xF8,
    0xF9, 0xFA,
];
