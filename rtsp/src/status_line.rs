use crate::{Emit, Parse, StatusCode, Version};
use core::fmt;
use nom::{bytes::complete::tag, character::complete::space0, IResult};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Default)]
pub struct StatusLine {
    pub version: Version,
    pub status_code: StatusCode,
}

impl StatusLine {
    pub fn new(version: Version, status_code: StatusCode) -> Self {
        StatusLine {
            version,
            status_code,
        }
    }
}

impl fmt::Display for StatusLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "RTSP/{} {}", self.version, self.status_code)
    }
}

impl Parse for StatusLine {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let (input, _) = tag("RTSP/")(input)?;
        let (input, version) = Version::parse(input)?;
        let (input, _) = space0(input)?;
        let (input, status_code) = StatusCode::parse(input)?;
        let (input, _) = space0(input)?;
        let (input, _) = tag("\r\n")(input)?;
        Ok((input, StatusLine::new(version, status_code)))
    }
}

impl<W: fmt::Write> Emit<W> for StatusLine {
    fn emit(&self, out: &mut W) -> fmt::Result {
        write!(out, "RTSP/{} {}\r\n", self.version, self.status_code)
    }
}

impl From<(Version, StatusCode)> for StatusLine {
    fn from(tuple: (Version, StatusCode)) -> Self {
        StatusLine::new(tuple.0, tuple.1)
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
        let buffer = "RTSP/1.0 200 OK\r\n";
        assert_eq!(
            StatusLine::parse(buffer),
            Ok((
                "",
                StatusLine::new(Version::new(1, 0), StatusCode::new(200))
            ))
        );
    }

    #[test]
    fn emit() {
        let mut buffer: String<U256> = String::new();
        let t = StatusLine::new(Version::new(1, 0), StatusCode::new(200));
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, "RTSP/1.0 200 OK\r\n");
    }
}
