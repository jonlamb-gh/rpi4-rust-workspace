//! Status code
//!
//! [RFC2326](https://tools.ietf.org/html/rfc2326#section-7.1.1)
//!
//! Copied from https://github.com/sgodwincs/rtsp-rs/blob/master/rtsp-2/src/status.rs

use crate::{Emit, Parse};
use core::fmt;
use nom::{
    bytes::complete::{take_till, take_till1},
    character::complete::space0,
    combinator::map_res,
    AsChar, IResult,
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct StatusCode(u16);

impl Default for StatusCode {
    fn default() -> StatusCode {
        StatusCode(100)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StatusCodeClass {
    /// A status code between [100, 199].
    Informational,

    /// A status code between [200, 299].
    Success,

    /// A status code between [300, 399].
    Redirection,

    /// A status code between [400, 499].
    ClientError,

    /// A status code between [500, 599].
    ServerError,
}

impl StatusCode {
    pub fn new(code: u16) -> Self {
        StatusCode(code)
    }

    pub fn class(self) -> StatusCodeClass {
        use self::StatusCodeClass::*;

        match u16::from(self) {
            100..=199 => Informational,
            200..=299 => Success,
            300..=399 => Redirection,
            400..=499 => ClientError,
            500..=599 => ServerError,
            _ => panic!("status code with invalid class"),
        }
    }

    pub fn is_client_error(self) -> bool {
        self.class() == StatusCodeClass::ClientError
    }

    pub fn is_informational(self) -> bool {
        self.class() == StatusCodeClass::Informational
    }

    pub fn is_redirection(self) -> bool {
        self.class() == StatusCodeClass::Redirection
    }

    pub fn is_server_error(self) -> bool {
        self.class() == StatusCodeClass::ServerError
    }

    pub fn is_success(self) -> bool {
        self.class() == StatusCodeClass::Success
    }

    pub fn canonical_reason(&self) -> Option<&str> {
        // TODO
        Some(match self.0 {
            100 => "Continue",
            200 => "OK",
            _ => return None,
        })
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:0>3} {}",
            self.0,
            self.canonical_reason().map_or("NA", |r| r)
        )
    }
}

impl From<u16> for StatusCode {
    fn from(val: u16) -> Self {
        StatusCode(val)
    }
}

impl From<StatusCode> for u16 {
    fn from(val: StatusCode) -> Self {
        val.0
    }
}

impl Parse for StatusCode {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let (input, status_code) = map_res(take_till1(|c: char| !c.is_dec_digit()), |s: &str| {
            s.parse::<u16>()
        })(input)?;
        let (input, _) = space0(input)?;
        let (input, _text) = take_till(|c: char| !c.is_alpha() && c != ' ')(input)?;
        Ok((input, StatusCode::new(status_code)))
    }
}

impl<W: fmt::Write> Emit<W> for StatusCode {
    fn emit(&self, out: &mut W) -> fmt::Result {
        write!(
            out,
            "{} {}",
            self.0,
            self.canonical_reason().map_or("NA", |r| r)
        )
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
        let buffer = "100";
        assert_eq!(StatusCode::parse(buffer), Ok(("", StatusCode::new(100))));
        let buffer = "200 OK";
        assert_eq!(StatusCode::parse(buffer), Ok(("", StatusCode::new(200))));
        let buffer = "407 Proxy Authentication Required";
        assert_eq!(StatusCode::parse(buffer), Ok(("", StatusCode::new(407))));
    }

    #[test]
    fn emit() {
        let mut buffer: String<U256> = String::new();
        let t = StatusCode::new(407);
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, "407 NA");
    }
}
