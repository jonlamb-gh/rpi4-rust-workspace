use core::fmt;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default)]
pub struct EthernetAddress(pub [u8; 6]);

impl EthernetAddress {
    pub const BROADCAST: EthernetAddress = EthernetAddress([0xff; 6]);
}

impl From<[u8; 6]> for EthernetAddress {
    fn from(octets: [u8; 6]) -> Self {
        EthernetAddress(octets)
    }
}

impl Into<[u8; 6]> for EthernetAddress {
    fn into(self) -> [u8; 6] {
        self.0
    }
}

impl fmt::Display for EthernetAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.0;
        write!(
            f,
            "{:02X}:{:02X}:{:02X}:{:02X}:{:02X}:{:02X}",
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
        )
    }
}
