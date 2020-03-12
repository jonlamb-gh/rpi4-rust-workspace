use crate::{Emit, HeaderString, Parse};
use core::fmt;
use core::ops::{Deref, DerefMut};
use nom::{
    bytes::complete::{tag, tag_no_case, take_till1, take_until},
    character::complete::space0,
    combinator::opt,
    AsChar, IResult,
};

// TODO - timeout as optional member
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Session(pub HeaderString);

impl From<&str> for Session {
    fn from(t: &str) -> Self {
        Session(HeaderString::from(t))
    }
}

impl From<HeaderString> for Session {
    fn from(t: HeaderString) -> Self {
        Session(t)
    }
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Session: {}", self.0)
    }
}

impl Parse for Session {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let (input, _) = tag_no_case("Session:")(input)?;
        let (input, _) = space0(input)?;
        let (input, session) = take_till1(|c: char| !c.is_alphanum())(input)?;
        let (input, _) = opt(take_until("\r\n"))(input)?;
        let (input, _) = opt(tag("\r\n"))(input)?;
        Ok((input, session.into()))
    }
}

impl<W: fmt::Write> Emit<W> for Session {
    fn emit(&self, out: &mut W) -> fmt::Result {
        write!(out, "Session: {}\r\n", self.0)
    }
}

impl Deref for Session {
    type Target = HeaderString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Session {
    fn deref_mut(&mut self) -> &mut Self::Target {
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
        let buffer = "Session: 1199687724;timeout=60\r\n";
        assert_eq!(Session::parse(buffer), Ok(("", "1199687724".into())));
        let buffer = "Session: 1199687721\r\n";
        assert_eq!(Session::parse(buffer), Ok(("", "1199687721".into())));
        let buffer = "Session: 1234abcd";
        assert_eq!(Session::parse(buffer), Ok(("", "1234abcd".into())));
    }

    #[test]
    fn emit() {
        let t: Session = "1199687724".into();
        let mut buffer: String<U256> = String::new();
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, "Session: 1199687724\r\n");
    }
}
