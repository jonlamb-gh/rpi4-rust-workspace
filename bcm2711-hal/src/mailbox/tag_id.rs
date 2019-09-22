use crate::mailbox::Error;
use core::convert::TryFrom;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TagId {
    Last = 0x0000_0000,
    SetCursorState = 0x0000_8011,
    GetSerialNum = 0x0001_0004,
    GetArmMem = 0x0001_0005,
    GetVcMem = 0x0001_0006,
    GetClockRate = 0x0003_0002,
    GetTemperature = 0x0003_0006,
    AllocBuffer = 0x0004_0001,
    GetPitch = 0x0004_0008,
    GetPhySize = 0x0004_0003,
    SetPhySize = 0x0004_8003,
    SetVirtSize = 0x0004_8004,
    SetDepth = 0x0004_8005,
    SetPixelOrder = 0x0004_8006,
    SetVirtOffset = 0x0004_8009,
}

impl From<TagId> for u32 {
    fn from(tag: TagId) -> u32 {
        tag as _
    }
}

impl TryFrom<u32> for TagId {
    type Error = Error;

    fn try_from(value: u32) -> Result<TagId, self::Error> {
        match value {
            0x0000_0000 => Ok(TagId::Last),
            0x0000_8011 => Ok(TagId::SetCursorState),
            0x0001_0004 => Ok(TagId::GetSerialNum),
            0x0001_0005 => Ok(TagId::GetArmMem),
            0x0001_0006 => Ok(TagId::GetVcMem),
            0x0003_0002 => Ok(TagId::GetClockRate),
            0x0003_0006 => Ok(TagId::GetTemperature),
            0x0004_0001 => Ok(TagId::AllocBuffer),
            0x0004_0008 => Ok(TagId::GetPitch),
            0x0004_0003 => Ok(TagId::GetPhySize),
            0x0004_8003 => Ok(TagId::SetPhySize),
            0x0004_8004 => Ok(TagId::SetVirtSize),
            0x0004_8005 => Ok(TagId::SetDepth),
            0x0004_8006 => Ok(TagId::SetPixelOrder),
            0x0004_8009 => Ok(TagId::SetVirtOffset),
            _ => Err(Error::UnkownTagId(value)),
        }
    }
}
