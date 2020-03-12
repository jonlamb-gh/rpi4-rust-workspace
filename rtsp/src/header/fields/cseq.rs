//! CSeq
//!
//! Copied from https://github.com/sgodwincs/rtsp-rs/blob/master/rtsp-2/src/header/types/cseq.rs
//!
//! [RFC2326](https://tools.ietf.org/html/rfc2326#section-12.17)

use crate::{Emit, Parse};
use core::convert::TryFrom;
use core::fmt;
use core::ops::{Add, Deref, Sub};
use nom::{
    bytes::complete::{tag_no_case, take_till1},
    character::complete::multispace1,
    combinator::map_res,
    AsChar, IResult,
};

/// The maximum size the CSeq can be.
pub const MAX_CSEQ: u32 = 999_999_999;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CSeq(u32);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CSeqError {
    /// The `"CSeq"` header was parsed, but the length exceeds the maximum
    /// length a CSeq can be.
    ExceedsMaximumLength,
}

impl CSeq {
    pub fn wrapping_increment(self) -> Self {
        CSeq((self.0 + 1) % (MAX_CSEQ + 1))
    }

    pub fn len(&self) -> usize {
        self.0 as usize
    }
}

impl fmt::Display for CSeq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CSeq: {}", self.0)
    }
}

impl Parse for CSeq {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let (input, _) = tag_no_case("CSeq:")(input)?;
        let (input, _) = multispace1(input)?;
        let (input, val) = map_res(take_till1(|c: char| !c.is_dec_digit()), |s: &str| {
            s.parse::<u32>()
        })(input)?;
        let (input, _) = multispace1(input)?;
        let val = CSeq::try_from(val)
            .map_err(|_| nom::Err::Failure((input, nom::error::ErrorKind::ParseTo)))?;
        Ok((input, val))
    }
}

impl<W: fmt::Write> Emit<W> for CSeq {
    fn emit(&self, out: &mut W) -> fmt::Result {
        write!(out, "CSeq: {}\r\n", self.0)
    }
}

impl Add for CSeq {
    type Output = CSeq;

    fn add(self, other: CSeq) -> Self::Output {
        CSeq((self.0 + other.0) % (MAX_CSEQ + 1))
    }
}

impl Deref for CSeq {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Sub for CSeq {
    type Output = CSeq;

    fn sub(self, other: CSeq) -> Self::Output {
        CSeq(if self >= other {
            self.0 - other.0
        } else {
            MAX_CSEQ - (other.0 - self.0)
        })
    }
}

impl TryFrom<u32> for CSeq {
    type Error = CSeqError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        if value > MAX_CSEQ {
            Err(CSeqError::ExceedsMaximumLength)
        } else {
            Ok(CSeq(value))
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use heapless::consts::*;
    use heapless::String;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_sub() {
        let cseq_1 = CSeq::try_from(50).unwrap();
        let cseq_2 = CSeq::try_from(100).unwrap();

        assert_eq!(*(cseq_1 - cseq_1), 0);
        assert_eq!(*(cseq_2 - cseq_1), 50);
        assert_eq!(*(cseq_1 - cseq_2), MAX_CSEQ - 50);
    }

    #[test]
    fn parse() {
        let buffer = "CSeq: 2\r\n";
        assert_eq!(CSeq::parse(buffer), Ok(("", CSeq(2))));
        let buffer = "cseq:     00001    \r\n";
        assert_eq!(CSeq::parse(buffer), Ok(("", CSeq(1))));
    }

    #[test]
    fn emit() {
        let mut buffer: String<U256> = String::new();
        let t = CSeq(2);
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, "CSeq: 2\r\n");
    }
}
