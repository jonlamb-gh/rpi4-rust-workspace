//! Version
//!
//! [RFC2326](https://tools.ietf.org/html/rfc2326#section-7.1)

use crate::{Emit, Parse};
use core::fmt;
use nom::{
    bytes::complete::{tag, take_till1},
    combinator::map_res,
    AsChar, IResult,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Version(u8, u8);

impl Default for Version {
    fn default() -> Self {
        Version(1, 0)
    }
}

impl Version {
    pub fn new(maj: u8, min: u8) -> Self {
        Version(maj, min)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.0, self.1)
    }
}

impl Parse for Version {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let (input, maj) = map_res(take_till1(|c: char| !c.is_dec_digit()), |s: &str| {
            s.parse::<u8>()
        })(input)?;
        let (input, _) = tag(".")(input)?;
        let (input, min) = map_res(take_till1(|c: char| !c.is_dec_digit()), |s: &str| {
            s.parse::<u8>()
        })(input)?;
        Ok((input, Version::new(maj, min)))
    }
}

impl<W: fmt::Write> Emit<W> for Version {
    fn emit(&self, out: &mut W) -> fmt::Result {
        write!(out, "{}.{}", self.0, self.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::consts::*;
    use heapless::String;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse() {
        let buffer = "0.0";
        assert_eq!(Version::parse(buffer), Ok(("", Version::new(0, 0))));
        let buffer = "1.0";
        assert_eq!(Version::parse(buffer), Ok(("", Version::new(1, 0))));
        let buffer = "2.1";
        assert_eq!(Version::parse(buffer), Ok(("", Version::new(2, 1))));
    }

    #[test]
    fn parse_invalid() {
        let buffer = "257.0";
        assert!(Version::parse(buffer).is_err());
        let buffer = "0.257";
        assert!(Version::parse(buffer).is_err());
        let buffer = "";
        assert!(Version::parse(buffer).is_err());
    }

    #[test]
    fn emit() {
        let mut buffer: String<U256> = String::new();
        let t = Version::new(2, 1);
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, "2.1");
    }
}
