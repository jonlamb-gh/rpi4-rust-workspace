//! RTSP request method
//!
//! [RFC2326](https://tools.ietf.org/html/rfc2326#section-10)
//!
//! Copied from https://github.com/sgodwincs/rtsp-rs/blob/master/rtsp-2/src/method.rs

use core::convert::AsRef;
use core::fmt;
use core::str;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum Method {
    Describe,
    Announce,
    GetParameter,
    Options,
    Pause,
    Play,
    Record,
    Redirect,
    Setup,
    SetParameter,
    Teardown,
}

impl Method {
    pub fn as_str(&self) -> &str {
        match self {
            Method::Describe => "DESCRIBE",
            Method::Announce => "ANNOUNCE",
            Method::GetParameter => "GET_PARAMETER",
            Method::Options => "OPTIONS",
            Method::Pause => "PAUSE",
            Method::Play => "PLAY",
            Method::Record => "RECORD",
            Method::Redirect => "REDIRECT",
            Method::Setup => "SETUP",
            Method::SetParameter => "SET_PARAMETER",
            Method::Teardown => "TEARDOWN",
        }
    }

    pub fn enumerate() -> &'static [Method] {
        use Method::*;
        &[
            Describe,
            Announce,
            GetParameter,
            Options,
            Pause,
            Play,
            Record,
            Redirect,
            Setup,
            SetParameter,
            Teardown,
        ]
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl AsRef<[u8]> for Method {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl AsRef<str> for Method {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq<[u8]> for Method {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_str().as_bytes().eq_ignore_ascii_case(other)
    }
}

impl PartialEq<Method> for [u8] {
    fn eq(&self, other: &Method) -> bool {
        self.eq_ignore_ascii_case(other.as_str().as_bytes())
    }
}

impl<'method> PartialEq<&'method [u8]> for Method {
    fn eq(&self, other: &&'method [u8]) -> bool {
        self.as_str().as_bytes().eq_ignore_ascii_case(other)
    }
}

impl<'method> PartialEq<Method> for &'method [u8] {
    fn eq(&self, other: &Method) -> bool {
        self.eq_ignore_ascii_case(other.as_str().as_bytes())
    }
}

impl PartialEq<str> for Method {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq_ignore_ascii_case(other)
    }
}

impl PartialEq<Method> for str {
    fn eq(&self, other: &Method) -> bool {
        self.eq_ignore_ascii_case(other.as_str())
    }
}

impl<'method> PartialEq<&'method str> for Method {
    fn eq(&self, other: &&'method str) -> bool {
        self.as_str().eq_ignore_ascii_case(other)
    }
}

impl<'method> PartialEq<Method> for &'method str {
    fn eq(&self, other: &Method) -> bool {
        self.eq_ignore_ascii_case(other.as_str())
    }
}

impl str::FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for m in Self::enumerate() {
            if m == s {
                return Ok(*m);
            }
        }
        Err(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::str::FromStr;

    #[test]
    fn from_str() {
        for m in Method::enumerate() {
            let s = m.as_str();
            assert_eq!(Method::from_str(s), Ok(*m));
        }
        assert_eq!(Method::from_str("DESCRIBE"), Ok(Method::Describe));
        assert_eq!(Method::from_str("describe"), Ok(Method::Describe));
    }
}
