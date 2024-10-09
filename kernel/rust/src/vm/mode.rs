use core::fmt::{Debug, Formatter};

#[allow(non_camel_case_types, reason = "it gives better representation")]
#[repr(usize)]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
#[allow(
    clippy::unusual_byte_groupings,
    reason = "it better represents the data"
)]
pub enum Mode {
    Table = 0b0_000_0,

    _R__ = 0b0_001_0,
    _RW_ = 0b0_011_0,
    ___X = 0b0_100_0,
    _R_X = 0b0_101_0,
    _RWX = 0b0_111_0,

    UR__ = 0b1_001_0,
    URW_ = 0b1_011_0,
    U__X = 0b1_100_0,
    UR_X = 0b1_101_0,
    URWX = 0b1_111_0,
}

pub struct InvalidEnumVariant(pub usize);

impl TryFrom<usize> for Mode {
    type Error = InvalidEnumVariant;

    #[allow(
        clippy::unusual_byte_groupings,
        reason = "it better represents the data"
    )]
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        let value = value & !0b00_111_0_000_1; // TODO: do not discard these bits
        match value {
            0b0_000_0 => Ok(Self::Table),

            0b0_001_0 => Ok(Self::_R__),
            0b0_011_0 => Ok(Self::_RW_),
            0b0_100_0 => Ok(Self::___X),
            0b0_101_0 => Ok(Self::_R_X),
            0b0_111_0 => Ok(Self::_RWX),

            0b1_001_0 => Ok(Self::UR__),
            0b1_011_0 => Ok(Self::URW_),
            0b1_100_0 => Ok(Self::U__X),
            0b1_101_0 => Ok(Self::UR_X),
            0b1_111_0 => Ok(Self::URWX),
            _ => Err(InvalidEnumVariant(value)),
        }
    }
}

impl Mode {
    #[allow(
        clippy::unusual_byte_groupings,
        reason = "it better represents the data"
    )]
    pub(crate) const VALID: usize = 0b0_000_1;
    pub(crate) const SHIFT: usize = 10;
    pub(crate) const ACCESS_MASK: usize = Self::URWX as usize;
}

impl Debug for InvalidEnumVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:#b}", self.0)
    }
}
