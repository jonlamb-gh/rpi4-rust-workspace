use crate::{Emit, Parse};
use core::fmt;
use heapless::consts::U256;
use heapless::String;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_till1, take_until},
    character::complete::multispace0,
    combinator::opt,
    IResult,
};

pub mod fields;
pub mod headers;
pub mod name;

pub use fields::*;
pub use headers::Headers;
pub use name::HeaderName;

pub type HeaderStringCapacity = U256;
pub type HeaderString = String<HeaderStringCapacity>;

#[derive(Clone, Debug, PartialEq)]
pub enum Header {
    CSeq(CSeq),
    Public(Public),
    Session(Session),
    Other(HeaderString, HeaderString),
}

impl From<CSeq> for Header {
    fn from(t: CSeq) -> Self {
        Header::CSeq(t)
    }
}

impl From<Public> for Header {
    fn from(t: Public) -> Self {
        Header::Public(t)
    }
}

impl From<Session> for Header {
    fn from(t: Session) -> Self {
        Header::Session(t)
    }
}

impl From<(HeaderString, HeaderString)> for Header {
    fn from(t: (HeaderString, HeaderString)) -> Self {
        Header::Other(t.0, t.1)
    }
}

impl From<(&str, &str)> for Header {
    fn from(t: (&str, &str)) -> Self {
        Header::Other(HeaderString::from(t.0), HeaderString::from(t.1))
    }
}

impl Header {
    pub fn name(&self) -> Option<HeaderName> {
        Some(match self {
            Header::CSeq(_) => HeaderName::CSeq,
            Header::Public(_) => HeaderName::Public,
            Header::Session(_) => HeaderName::Session,
            // TODO - not sure about HeadName stuff yet
            Header::Other(_k, _v) => return None,
        })
    }
}

impl Parse for Header {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let (input, _) = opt(tag("\r\n"))(input)?;
        let (input, header) = header_alt(input)?;
        Ok((input, header))
    }
}

fn header_alt(input: &str) -> IResult<&str, Header> {
    let (input, header) = alt((cseq, public, session, other))(input)?;
    Ok((input, header))
}

fn cseq(input: &str) -> IResult<&str, Header> {
    let (input, h) = CSeq::parse(input)?;
    Ok((input, Header::CSeq(h)))
}

fn public(input: &str) -> IResult<&str, Header> {
    let (input, h) = Public::parse(input)?;
    Ok((input, Header::Public(h)))
}

fn session(input: &str) -> IResult<&str, Header> {
    let (input, h) = Session::parse(input)?;
    Ok((input, Header::Session(h)))
}

fn other(input: &str) -> IResult<&str, Header> {
    let (input, key) = take_till1(|c: char| c == ':')(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, val) = take_until("\r\n")(input)?;
    let (input, _) = tag("\r\n")(input)?;
    Ok((input, Header::from((key, val))))
}

impl<W: fmt::Write> Emit<W> for Header {
    fn emit(&self, out: &mut W) -> fmt::Result {
        match self {
            Header::CSeq(t) => t.emit(out),
            Header::Public(t) => t.emit(out),
            Header::Session(t) => t.emit(out),
            Header::Other(k, v) => write!(out, "{}: {}\r\n", k, v),
        }
    }
}
