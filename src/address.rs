use core::fmt::{Error, Formatter};

/// A 64-bit address of a device. These are globally unique, and used to single out a single device on
/// a potentially crowded bus
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Address(pub u64);

impl Address {
    pub fn family_code(&self) -> u8 {
        self.0.to_le_bytes()[0]
    }
}

impl core::fmt::Debug for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:016X?}", self.0)
    }
}
