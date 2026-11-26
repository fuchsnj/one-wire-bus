//! 1-Wire ROM and power-supply command bytes.

/// Searches for devices on the bus.
pub const SEARCH_NORMAL: u8 = 0xF0;
/// Selects one device by its ROM address.
pub const MATCH_ROM: u8 = 0x55;
/// Selects all devices without sending a ROM address.
pub const SKIP_ROM: u8 = 0xCC;
/// Searches for devices with an active alarm.
pub const SEARCH_ALARM: u8 = 0xEC;
/// Queries whether selected devices use parasite power.
pub const READ_POWER_SUPPLY: u8 = 0xB4;
