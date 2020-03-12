use crate::header::fields::*;
use crate::{Emit, Header, HeaderString, Parse};
use core::fmt;
use heapless::consts::U16;
use heapless::Vec;
use nom::{
    bytes::complete::tag,
    combinator::{iterator, opt},
    IResult,
};

// TODO - unify errors
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    Capacity,
}

pub type HeadersCapacity = U16;

#[derive(Clone, Debug, PartialEq)]
pub struct Headers(pub Vec<Header, HeadersCapacity>);

impl Default for Headers {
    fn default() -> Headers {
        Headers::new()
    }
}

impl Headers {
    pub fn new() -> Headers {
        Headers(Vec::new())
    }

    pub fn push(&mut self, h: Header) -> Result<(), Error> {
        self.0.push(h).map_err(|_| Error::Capacity)?;
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Header> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Header> {
        self.0.iter_mut()
    }

    pub fn cseq(&self) -> Option<CSeq> {
        for h in &self.0 {
            if let Header::CSeq(t) = h {
                return Some(*t);
            }
        }
        None
    }

    pub fn session(&self) -> Option<&HeaderString> {
        for h in &self.0 {
            if let Header::Session(t) = h {
                return Some(t);
            }
        }
        None
    }
}

impl Parse for Headers {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let mut hdrs = Headers::new();
        let mut it = iterator(input, Header::parse);
        let r = it.map(|v| hdrs.push(v)).collect::<Result<(), Error>>();
        r.map_err(|_| nom::Err::Failure((input, nom::error::ErrorKind::ParseTo)))?;
        let (input, _) = it.finish()?;
        let (input, _) = opt(tag("\r\n"))(input)?;
        Ok((input, hdrs))
    }
}

impl<W: fmt::Write> Emit<W> for Headers {
    fn emit(&self, out: &mut W) -> fmt::Result {
        if let Some(cseq) = self.cseq() {
            cseq.emit(out)?;
        }
        for h in self.0.iter() {
            match h {
                Header::CSeq(_) => (),
                _ => h.emit(out)?,
            }
        }
        Ok(())
    }
}
