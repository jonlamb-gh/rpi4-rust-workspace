use crate::{Emit, Method, Parse, Uri, Version};
use core::fmt;
use core::str::FromStr;
use nom::{
    bytes::complete::{tag, take_till1, take_until},
    character::complete::space0,
    IResult,
};

#[derive(Clone, Debug, PartialEq)]
pub struct RequestLine {
    pub method: Method,
    pub uri: Option<Uri>,
    pub version: Version,
}

impl RequestLine {
    pub fn new(method: Method, uri: Option<Uri>, version: Version) -> Self {
        RequestLine {
            method,
            uri,
            version,
        }
    }
}

impl fmt::Display for RequestLine {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ", self.method.as_str())?;
        if let Some(uri) = &self.uri {
            write!(f, "{} ", uri)?;
        }
        write!(f, "RTSP/{}", self.version)
    }
}

impl Parse for RequestLine {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let (input, method_str) = take_till1(|c: char| c == ' ')(input)?;
        let method = Method::from_str(method_str)
            .map_err(|_| nom::Err::Failure((input, nom::error::ErrorKind::ParseTo)))?;
        let (input, maybe_uri) = take_until(" RTSP")(input)?;
        let (maybe_uri, _) = space0(maybe_uri)?;
        let uri = if maybe_uri.len() != 0 {
            Some(Uri::from(maybe_uri))
        } else {
            None
        };
        let (input, _) = space0(input)?;
        let (input, _) = tag("RTSP/")(input)?;
        let (input, version) = Version::parse(input)?;
        let (input, _) = tag("\r\n")(input)?;
        Ok((input, RequestLine::new(method, uri, version)))
    }
}

impl<W: fmt::Write> Emit<W> for RequestLine {
    fn emit(&self, out: &mut W) -> fmt::Result {
        write!(out, "{} ", self.method.as_str())?;
        if let Some(uri) = &self.uri {
            write!(out, "{} ", uri)?;
        }
        write!(out, "RTSP/{}\r\n", self.version)
    }
}

impl From<(Method, Option<Uri>, Version)> for RequestLine {
    fn from(tuple: (Method, Option<Uri>, Version)) -> Self {
        RequestLine::new(tuple.0, tuple.1, tuple.2)
    }
}

impl From<(Method, Uri, Version)> for RequestLine {
    fn from(tuple: (Method, Uri, Version)) -> Self {
        RequestLine::new(tuple.0, Some(tuple.1), tuple.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::consts::*;
    use heapless::String;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_without_uri() {
        let buffer = "PLAY RTSP/1.0\r\n";
        assert_eq!(
            RequestLine::parse(buffer),
            Ok(("", RequestLine::new(Method::Play, None, Version::new(1, 0))))
        );
    }

    #[test]
    fn parse_with_uri() {
        let buffer = "OPTIONS rtsp://192.168.1.123:554/streaming/channels/1 RTSP/1.2\r\n";
        assert_eq!(
            RequestLine::parse(buffer),
            Ok((
                "",
                RequestLine::new(
                    Method::Options,
                    Some(Uri::from("rtsp://192.168.1.123:554/streaming/channels/1")),
                    Version::new(1, 2)
                )
            ))
        );
    }

    #[test]
    fn emit_without_uri() {
        let mut buffer: String<U256> = String::new();
        let t = RequestLine::new(Method::Play, None, Version::new(1, 0));
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, "PLAY RTSP/1.0\r\n");
    }

    #[test]
    fn emit_with_uri() {
        let mut buffer: String<U256> = String::new();
        let t = RequestLine::new(
            Method::Play,
            Some(Uri::from(
                "rtsp://192.168.1.122:554/streaming/channels/1/trackID=1",
            )),
            Version::new(1, 0),
        );
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(
            buffer,
            "PLAY rtsp://192.168.1.122:554/streaming/channels/1/trackID=1 RTSP/1.0\r\n"
        );
    }
}
