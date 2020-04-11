/// [RFC3984](https://tools.ietf.org/html/rfc3984#section-5.2)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum NalUnitType {
    /// [RFC3984](https://tools.ietf.org/html/rfc3984#section-5.6)
    /// Type 1
    SingleNalUnit,
    /// 6
    Sei,
    /// 7
    Sps,
    /// 8
    Pps,
    /// [RFC2435](https://tools.ietf.org/html/rfc2435#section-5.8)
    /// Type 28
    FuA,
    /// Everything else
    Unknown(u8),
}

impl From<u8> for NalUnitType {
    fn from(val: u8) -> Self {
        match val {
            1 => NalUnitType::SingleNalUnit,
            6 => NalUnitType::Sei,
            7 => NalUnitType::Sps,
            8 => NalUnitType::Pps,
            28 => NalUnitType::FuA,
            _ => NalUnitType::Unknown(val),
        }
    }
}

impl Into<u8> for NalUnitType {
    fn into(self) -> u8 {
        match self {
            NalUnitType::SingleNalUnit => 1,
            NalUnitType::Sei => 6,
            NalUnitType::Sps => 7,
            NalUnitType::Pps => 8,
            NalUnitType::FuA => 28,
            NalUnitType::Unknown(v) => v,
        }
    }
}
