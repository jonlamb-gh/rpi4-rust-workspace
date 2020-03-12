//! Header name
//!
//! [RFC2326](https://tools.ietf.org/html/rfc2326#section-12)
//!
//! Copied from https://github.com/sgodwincs/rtsp-rs/blob/master/rtsp-2/src/header/name.rs

use core::convert::AsRef;
use core::fmt;
use core::str;

macro_rules! standard_headers {
    (
        $(
            $(#[$docs:meta])*
            ($variant:ident, $name:expr, $canonical_name:expr);
        )+
    ) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        #[non_exhaustive]
        pub enum HeaderName {
        $(
            $(#[$docs])*
            $variant,
        )+
        }

         impl HeaderName {
            pub fn as_str(&self) -> &str {
                use self::HeaderName::*;

                match *self {
                $(
                    $variant => $name,
                )+
                }
            }

             pub fn canonical_name(&self) -> &str {
                use self::HeaderName::*;

                match *self {
                $(
                    $variant => $canonical_name,
                )+
                }
            }
        }

        #[cfg(test)]
        mod test {
            use crate::header::name::HeaderName;

            #[test]
            fn test_standard_header_as_str() {
            $(
                let header_name = HeaderName::$variant;
                assert_eq!(header_name.as_str(), $name);
            )+
            }

            #[test]
            fn test_standard_header_canonical_name() {
            $(
                let header_name = HeaderName::$variant;
                assert_eq!(header_name.canonical_name(), $canonical_name);
            )+
            }

            #[test]
            fn test_standard_header_name_equality() {
            $(
                let header_name = HeaderName::$variant;
                assert_eq!(
                    header_name.as_str(),
                    header_name.canonical_name().to_lowercase().as_str()
                );
            )+
            }
        }
    }
}

impl AsRef<[u8]> for HeaderName {
    fn as_ref(&self) -> &[u8] {
        self.as_str().as_bytes()
    }
}

impl AsRef<str> for HeaderName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.canonical_name())
    }
}

impl PartialEq<[u8]> for HeaderName {
    fn eq(&self, other: &[u8]) -> bool {
        self.as_str().as_bytes().eq_ignore_ascii_case(other)
    }
}

impl PartialEq<HeaderName> for [u8] {
    fn eq(&self, other: &HeaderName) -> bool {
        self.eq_ignore_ascii_case(other.as_str().as_bytes())
    }
}

impl<'header> PartialEq<&'header [u8]> for HeaderName {
    fn eq(&self, other: &&'header [u8]) -> bool {
        self.as_str().as_bytes().eq_ignore_ascii_case(other)
    }
}

impl<'header> PartialEq<HeaderName> for &'header [u8] {
    fn eq(&self, other: &HeaderName) -> bool {
        self.eq_ignore_ascii_case(other.as_str().as_bytes())
    }
}

impl PartialEq<str> for HeaderName {
    fn eq(&self, other: &str) -> bool {
        self.as_str().eq_ignore_ascii_case(other)
    }
}

impl PartialEq<HeaderName> for str {
    fn eq(&self, other: &HeaderName) -> bool {
        self.eq_ignore_ascii_case(other.as_str())
    }
}

impl<'header> PartialEq<&'header str> for HeaderName {
    fn eq(&self, other: &&'header str) -> bool {
        self.as_str().eq_ignore_ascii_case(other)
    }
}

impl<'header> PartialEq<HeaderName> for &'header str {
    fn eq(&self, other: &HeaderName) -> bool {
        self.eq_ignore_ascii_case(other.as_str())
    }
}

standard_headers! {
    /// Accept
    /// [[RFC2326, Section 12.1](https://tools.ietf.org/html/rfc2326#section-12.1)]
    (Accept, "accept", "Accept");

    /// Allow
    /// [[RFC2326, Section 12.4](https://tools.ietf.org/html/rfc2326#section-12.4)]
    (Allow, "allow", "Allow");

    /// Authorization
    /// [[RFC2326, Section 12.5](https://tools.ietf.org/html/rfc2326#section-12.5)]
    (Authorization, "authorization", "Authorization");

    /// Bandwidth
    /// [[RFC2326, Section 12.6](https://tools.ietf.org/html/rfc2326#section-12.6)]
    (Bandwidth, "bandwidth", "Bandwidth");

    /// Blocksize
    (Blocksize, "blocksize", "Blocksize");

    /// Cache-Control
    (CacheControl, "cache-control", "Cache-Control");

    /// Connection
    (Connection, "connection", "Connection");

    /// Content-Base
    (ContentBase, "content-base", "Content-Base");

    /// Content-Encoding
    (ContentEncoding, "content-encoding", "Content-Encoding");

    /// Content-Language
    (ContentLanguage, "content-language", "Content-Language");

    /// Content-Length
    (ContentLength, "content-length", "Content-Length");

    /// Content-Location
    (ContentLocation, "content-location", "Content-Location");

    /// Content-Type
    (ContentType, "content-type", "Content-Type");

    /// CSeq
    (CSeq, "cseq", "CSeq");

    /// Date
    (Date, "date", "Date");

    /// Expires
    (Expires, "expires", "Expires");

    /// From
    (From, "from", "From");

    /// Location
    (Location, "location", "Location");

    /// Proxy-Authenticate
    (ProxyAuthenticate, "proxy-authenticate", "Proxy-Authenticate");

    /// Proxy-Require
    (ProxyRequire, "proxy-require", "Proxy-Require");

    /// Public
    (Public, "public", "Public");

    /// Range
    (Range, "range", "Range");

    /// Referrer
    (Referrer, "referrer", "Referrer");

    /// Require
    (Require, "require", "Require");

    /// Retry-After
    (RetryAfter, "retry-after", "Retry-After");

    /// RTP-Info
    (RTPInfo, "rtp-info", "RTP-Info");

    /// Scale
    (Scale, "scale", "Scale");

    /// Speed
    (Speed, "speed", "Speed");

    /// Session
    (Session, "session", "Session");

    /// Timestamp
    (Timestamp, "timestamp", "Timestamp");

    /// Transport
    (Transport, "transport", "Transport");

    /// Unsupported
    (Unsupported, "unsupported", "Unsupported");

    /// User-Agent
    (UserAgent, "user-agent", "User-Agent");

    /// Via
    (Via, "via", "Via");

    /// WWW-Authenticate
    /// [[RFC2326, Section 12.44](https://tools.ietf.org/html/rfc2326#section-12.44)]
    (WWWAuthenticate, "www-authenticate", "WWW-Authenticate");
}
