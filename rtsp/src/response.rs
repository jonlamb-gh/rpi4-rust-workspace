//! Response
//!
//! [RFC2326](https://tools.ietf.org/html/rfc2326#section-7)

use crate::{Emit, Headers, Parse, StatusLine};
use core::fmt;
use nom::IResult;

// TODO - flatten status line fields
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Response {
    pub status_line: StatusLine,
    pub headers: Headers,
    // body, u8/str?
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Response {}", self.status_line)
    }
}

impl Parse for Response {
    type Type = Self;

    fn parse(input: &str) -> IResult<&str, Self::Type> {
        let (input, status_line) = StatusLine::parse(input)?;
        let (input, headers) = Headers::parse(input)?;
        Ok((
            input,
            Response {
                status_line,
                headers,
            },
        ))
    }
}

impl<W: fmt::Write> Emit<W> for Response {
    fn emit(&self, out: &mut W) -> fmt::Result {
        self.status_line.emit(out)?;
        self.headers.emit(out)?;
        write!(out, "\r\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::header::fields::*;
    use crate::{Method, StatusCode, Version};
    use core::convert::TryFrom;
    use heapless::consts::*;
    use heapless::String;
    use pretty_assertions::assert_eq;

    const OPTIONS_RESP: &str = "RTSP/1.0 200 OK\r\n\
CSeq: 2\r\n\
Public: OPTIONS, DESCRIBE, PLAY, PAUSE, SETUP, TEARDOWN, SET_PARAMETER, GET_PARAMETER\r\n\
Date: Fri, Jan 02 1970 23:34:03 GMT\r\n\
\r\n";

    const DESCRIBE_RESP: &str = "RTSP/1.0 200 OK\r\n\
CSeq: 3\r\n\
Content-type: application/sdp\r\n\
Content-Base: rtsp://192.168.1.111:554/streaming/channels/1/\r\n\
Content-length: 543\r\n\
\r\n";

    const SETUP_RESP: &str = "RTSP/1.0 200 OK\r\n\
CSeq: 4\r\n\
Session: 1199687724;timeout=60\r\n\
Transport: RTP/AVP;unicast;client_port=49154-49155;server_port=8212-8213;ssrc=544c26bf;mode=\"play\"\r\n\
Date: Sun, Jan 04 1970 08:24:43 GMT\r\n\
\r\n";

    const PLAY_RESP: &str = "RTSP/1.0 200 OK\r\n\
CSeq: 5\r\n\
Session: 1199687724\r\n\
RTP-Info: url=rtsp://192.168.1.222:554/streaming/channels/1/trackID=1;seq=35105;rtptime=3111592\r\n\
Date: Sun, Jan 04 1970 08:24:43 GMT\r\n\
\r\n";

    fn headers_for_options() -> Headers {
        let public = Public(
            [
                Method::Options,
                Method::Describe,
                Method::Play,
                Method::Pause,
                Method::Setup,
                Method::Teardown,
                Method::SetParameter,
                Method::GetParameter,
            ]
            .iter()
            .cloned()
            .collect(),
        );
        Headers(
            [
                CSeq::try_from(2_u32).unwrap().into(),
                public.into(),
                ("Date", "Fri, Jan 02 1970 23:34:03 GMT").into(),
            ]
            .iter()
            .cloned()
            .collect(),
        )
    }

    fn headers_for_describe() -> Headers {
        Headers(
            [
                CSeq::try_from(3_u32).unwrap().into(),
                ("Content-type", "application/sdp").into(),
                (
                    "Content-Base",
                    "rtsp://192.168.1.111:554/streaming/channels/1/",
                )
                    .into(),
                ("Content-length", "543").into(),
            ]
            .iter()
            .cloned()
            .collect(),
        )
    }

    fn headers_for_setup() -> Headers {
        Headers(
            [
                CSeq::try_from(4_u32).unwrap().into(),
                Session::from("1199687724;timeout=60").into(),
                (
                    "Transport",
                    "RTP/AVP;unicast;client_port=49154-49155;server_port=8212-8213;ssrc=544c26bf;mode=\"play\"",
                )
                    .into(),
                ("Date", "Sun, Jan 04 1970 08:24:43 GMT").into(),
            ]
            .iter()
            .cloned()
            .collect(),
        )
    }

    fn headers_for_play() -> Headers {
        Headers(
            [
                CSeq::try_from(5_u32).unwrap().into(),
                Session::from("1199687724").into(),
                (
                    "RTP-Info",
                    "url=rtsp://192.168.1.222:554/streaming/channels/1/trackID=1;seq=35105;rtptime=3111592",
                )
                    .into(),
                ("Date", "Sun, Jan 04 1970 08:24:43 GMT").into(),
            ]
            .iter()
            .cloned()
            .collect(),
        )
    }

    #[test]
    fn parse_options() {
        let hdrs = headers_for_options();
        assert_eq!(
            Response::parse(OPTIONS_RESP),
            Ok((
                "",
                Response {
                    status_line: (Version::new(1, 0), StatusCode::new(200)).into(),
                    headers: hdrs,
                }
            ))
        );
    }

    #[test]
    fn emit_options() {
        let hdrs = headers_for_options();
        let t = Response {
            status_line: (Version::new(1, 0), StatusCode::new(200)).into(),
            headers: hdrs,
        };
        let mut buffer: String<U256> = String::new();
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, OPTIONS_RESP);
    }

    #[test]
    fn parse_describe() {
        let hdrs = headers_for_describe();
        assert_eq!(
            Response::parse(DESCRIBE_RESP),
            Ok((
                "",
                Response {
                    status_line: (Version::new(1, 0), StatusCode::new(200)).into(),
                    headers: hdrs,
                }
            ))
        );
    }

    #[test]
    fn emit_describe() {
        let hdrs = headers_for_describe();
        let t = Response {
            status_line: (Version::new(1, 0), StatusCode::new(200)).into(),
            headers: hdrs,
        };
        let mut buffer: String<U256> = String::new();
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, DESCRIBE_RESP);
    }

    #[test]
    fn parse_setup() {
        let hdrs = headers_for_setup();
        assert_eq!(
            Response::parse(SETUP_RESP),
            Ok((
                "",
                Response {
                    status_line: (Version::new(1, 0), StatusCode::new(200)).into(),
                    headers: hdrs,
                }
            ))
        );
    }

    #[test]
    fn emit_setup() {
        let hdrs = headers_for_setup();
        let t = Response {
            status_line: (Version::new(1, 0), StatusCode::new(200)).into(),
            headers: hdrs,
        };
        let mut buffer: String<U256> = String::new();
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, SETUP_RESP);
    }

    #[test]
    fn parse_play() {
        let hdrs = headers_for_play();
        assert_eq!(
            Response::parse(PLAY_RESP),
            Ok((
                "",
                Response {
                    status_line: (Version::new(1, 0), StatusCode::new(200)).into(),
                    headers: hdrs,
                }
            ))
        );
    }

    #[test]
    fn emit_play() {
        let hdrs = headers_for_play();
        let t = Response {
            status_line: (Version::new(1, 0), StatusCode::new(200)).into(),
            headers: hdrs,
        };
        let mut buffer: String<U256> = String::new();
        assert_eq!(t.emit(&mut buffer), Ok(()));
        assert_eq!(buffer, PLAY_RESP);
    }
}
