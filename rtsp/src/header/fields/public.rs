use crate::method::Method;
use crate::{Emit, Parse};
use core::fmt;
use core::ops::{Deref, DerefMut};
use core::str::FromStr;
use heapless::consts::U16;
use heapless::Vec;
use nom::{
    bytes::complete::{tag, tag_no_case, take_till1},
    character::complete::{line_ending, space0},
    combinator::{iterator, opt},
    AsChar, IResult,
};

// TODO - unify errors
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Error {
    Capacity,
}

pub type Capacity = U16;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Public(pub Vec<Method, Capacity>);

impl Public {
    pub fn new() -> Public {
        Public::default()
    }

    pub fn push(&mut self, m: Method) -> Result<(), Error> {
        self.0.push(m).map_err(|_| Error::Capacity)?;
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &Method> {
        self.0.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Method> {
        self.0.iter_mut()
    }
}

impl fmt::Display for Public {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Public: ")?;
        let last = self.0.len().saturating_sub(1);
        for (idx, m) in self.0.iter().enumerate() {
            if idx == last {
                write!(f, "{}", m.as_str())?;
            } else {
                write!(f, "{}, ", m.as_str())?;
            }
        }
        Ok(())
    }
}

impl Parse for Public {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let (input, _) = tag_no_case("Public:")(input)?;
        let (input, _) = space0(input)?;
        let mut public = Public::new();
        let mut it = iterator(input, parse_method);
        let r = it.map(|v| public.push(v)).collect::<Result<(), Error>>();
        r.map_err(|_| nom::Err::Failure((input, nom::error::ErrorKind::ParseTo)))?;
        let (input, _) = it.finish()?;
        let (input, _) = opt(line_ending)(input)?;
        Ok((input, public))
    }
}

fn parse_method(input: &str) -> IResult<&str, Method> {
    let (input, _) = space0(input)?;
    let (input, method_str) = take_till1(|c: char| !c.is_alpha() && c != '_')(input)?;
    let (input, _) = opt(tag(","))(input)?;
    let method = Method::from_str(method_str)
        .map_err(|_| nom::Err::Failure((input, nom::error::ErrorKind::ParseTo)))?;
    Ok((input, method))
}

impl<W: fmt::Write> Emit<W> for Public {
    fn emit(&self, out: &mut W) -> fmt::Result {
        write!(out, "Public: ")?;
        let last = self.0.len().saturating_sub(1);
        for (idx, m) in self.0.iter().enumerate() {
            if idx == last {
                write!(out, "{}", m.as_str())?;
            } else {
                write!(out, "{}, ", m.as_str())?;
            }
        }
        write!(out, "\r\n")
    }
}

impl Deref for Public {
    type Target = Vec<Method, Capacity>;

    fn deref(&self) -> &Vec<Method, Capacity> {
        &self.0
    }
}

impl DerefMut for Public {
    fn deref_mut(&mut self) -> &mut Vec<Method, Capacity> {
        &mut self.0
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use heapless::consts::*;
    use heapless::String;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse() {
        let buffer = "Public: OPTIONS, DESCRIBE, PLAY, PAUSE, SETUP, TEARDOWN, SET_PARAMETER, GET_PARAMETER\r\n";

        let a = [
            Method::Options,
            Method::Describe,
            Method::Play,
            Method::Pause,
            Method::Setup,
            Method::Teardown,
            Method::SetParameter,
            Method::GetParameter,
        ];
        let v: Vec<Method, Capacity> = a.iter().cloned().collect();
        assert_eq!(Public::parse(buffer), Ok(("", Public(v))));
    }

    #[test]
    fn emit() {
        let a = [
            Method::Options,
            Method::Describe,
            Method::Play,
            Method::Pause,
            Method::Setup,
            Method::Teardown,
            Method::SetParameter,
            Method::GetParameter,
        ];
        let v: Vec<Method, Capacity> = a.iter().cloned().collect();
        let t = Public(v);
        let mut buffer: String<U256> = String::new();
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, "Public: OPTIONS, DESCRIBE, PLAY, PAUSE, SETUP, TEARDOWN, SET_PARAMETER, GET_PARAMETER\r\n");
    }
}
