//! Request
//!
//! [RFC2326](https://tools.ietf.org/html/rfc2326#section-6)

use crate::{Emit, Headers, Parse, RequestLine};
use core::fmt;
use nom::IResult;

// TODO - flatten request line fields
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub request_line: RequestLine,
    pub headers: Headers,
    // body, u8/str?
}

impl fmt::Display for Request {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Request {}", self.request_line)
    }
}

impl Parse for Request {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let (input, request_line) = RequestLine::parse(input)?;
        let (input, headers) = Headers::parse(input)?;
        Ok((
            input,
            Request {
                request_line,
                headers,
            },
        ))
    }
}

impl<W: fmt::Write> Emit<W> for Request {
    fn emit(&self, out: &mut W) -> fmt::Result {
        self.request_line.emit(out)?;
        self.headers.emit(out)?;
        write!(out, "\r\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::fields::*;
    use crate::{Method, Uri, Version};
    use core::convert::TryFrom;
    use heapless::consts::*;
    use heapless::String;
    use pretty_assertions::assert_eq;

    const OPTIONS_REQ: &str = "OPTIONS rtsp://192.168.1.123:554/streaming/channels/1 RTSP/1.0\r\n\
CSeq: 1\r\n\
\r\n";

    const DESCRIBE_REQ: &str =
        "DESCRIBE rtsp://192.168.1.111:554/streaming/channels/1 RTSP/1.0\r\n\
CSeq: 2\r\n\
Accept: application/sdp\r\n\
\r\n";

    const SETUP_REQ: &str =
        "SETUP rtsp://192.168.1.122:554/streaming/channels/1/trackID=1 RTSP/1.0\r\n\
CSeq: 3\r\n\
Transport: RTP/AVP;unicast;client_port=49154-49155\r\n\
\r\n";

    const PLAY_REQ: &str = "PLAY rtsp://192.168.1.211:554/streaming/channels/1/ RTSP/1.0\r\n\
CSeq: 4\r\n\
Session: 1199687724\r\n\
\r\n";

    fn request_for_options() -> Request {
        Request {
            request_line: (
                Method::Options,
                Uri::from("rtsp://192.168.1.123:554/streaming/channels/1"),
                Version::new(1, 0),
            )
                .into(),
            headers: Headers(
                [CSeq::try_from(1_u32).unwrap().into()]
                    .iter()
                    .cloned()
                    .collect(),
            ),
        }
    }

    fn request_for_describe() -> Request {
        Request {
            request_line: (
                Method::Describe,
                Uri::from("rtsp://192.168.1.111:554/streaming/channels/1"),
                Version::new(1, 0),
            )
                .into(),
            headers: Headers(
                [
                    CSeq::try_from(2_u32).unwrap().into(),
                    ("Accept", "application/sdp").into(),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
        }
    }

    fn request_for_setup() -> Request {
        Request {
            request_line: (
                Method::Setup,
                Uri::from("rtsp://192.168.1.122:554/streaming/channels/1/trackID=1"),
                Version::new(1, 0),
            )
                .into(),
            headers: Headers(
                [
                    CSeq::try_from(3_u32).unwrap().into(),
                    ("Transport", "RTP/AVP;unicast;client_port=49154-49155").into(),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
        }
    }

    fn request_for_play() -> Request {
        Request {
            request_line: (
                Method::Play,
                Uri::from("rtsp://192.168.1.211:554/streaming/channels/1/"),
                Version::new(1, 0),
            )
                .into(),
            headers: Headers(
                [
                    CSeq::try_from(4_u32).unwrap().into(),
                    Session::from("1199687724").into(),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
        }
    }

    #[test]
    fn parse_options() {
        assert_eq!(Request::parse(OPTIONS_REQ), Ok(("", request_for_options())));
    }

    #[test]
    fn emit_options() {
        let t = request_for_options();
        let mut buffer: String<U256> = String::new();
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, OPTIONS_REQ);
    }

    #[test]
    fn parse_describe() {
        assert_eq!(
            Request::parse(DESCRIBE_REQ),
            Ok(("", request_for_describe()))
        );
    }

    #[test]
    fn emit_describe() {
        let t = request_for_describe();
        let mut buffer: String<U256> = String::new();
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, DESCRIBE_REQ);
    }

    #[test]
    fn parse_setup() {
        assert_eq!(Request::parse(SETUP_REQ), Ok(("", request_for_setup())));
    }

    #[test]
    fn emit_setup() {
        let t = request_for_setup();
        let mut buffer: String<U256> = String::new();
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, SETUP_REQ);
    }

    #[test]
    fn parse_play() {
        assert_eq!(Request::parse(PLAY_REQ), Ok(("", request_for_play())));
    }

    #[test]
    fn emit_play() {
        let t = request_for_play();
        let mut buffer: String<U256> = String::new();
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, PLAY_REQ);
    }
}
