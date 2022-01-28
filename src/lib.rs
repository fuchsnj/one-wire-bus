#![cfg_attr(not(feature = "std"), no_std)]

use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::{InputPin, OutputPin};

mod address;
pub mod commands;
pub mod crc;
mod error;

pub use address::Address;
pub use error::{OneWireError, OneWireResult};

pub const READ_SLOT_DURATION_MICROS: u16 = 70;

/// Implementation of the 1-Wire protocol.
/// https://www.maximintegrated.com/en/design/technical-documents/app-notes/1/126.html
#[derive(Debug)]
pub struct SearchState {
    // The address of the last found device
    address: u64,

    // bitflags of discrepancies found
    discrepancies: u64,

    // index of the last (leftmost / closest to MSB) discrepancy bit. This can be calculated from the
    // discrepancy bitflags, but it's cheaper to just save it. Index is an offset from the LSB
    last_discrepancy_index: u8,
}

pub struct OneWire<T> {
    pin: T,
}

impl<T, E> OneWire<T>
where
    T: InputPin<Error = E>,
    T: OutputPin<Error = E>,
{
    pub fn new(pin: T) -> OneWireResult<OneWire<T>, E> {
        let mut one_wire = OneWire { pin };
        // Pin should be high during idle.
        one_wire.release_bus()?;
        Ok(one_wire)
    }

    pub fn into_inner(self) -> T {
        self.pin
    }

    /// Disconnects the bus, letting another device (or the pull-up resistor) set the bus value
    pub fn release_bus(&mut self) -> OneWireResult<(), E> {
        self.pin
            .set_high()
            .map_err(|err| OneWireError::PinError(err))
    }

    /// Drives the bus low
    pub fn set_bus_low(&mut self) -> OneWireResult<(), E> {
        self.pin
            .set_low()
            .map_err(|err| OneWireError::PinError(err))
    }

    pub fn is_bus_high(&self) -> OneWireResult<bool, E> {
        self.pin
            .is_high()
            .map_err(|err| OneWireError::PinError(err))
    }

    pub fn is_bus_low(&self) -> OneWireResult<bool, E> {
        self.pin.is_low().map_err(|err| OneWireError::PinError(err))
    }

    fn wait_for_high(&self, delay: &mut impl DelayUs<u16>) -> OneWireResult<(), E> {
        // wait up to 250 µs for the bus to become high (from the pull-up resistor)
        for _ in 0..125 {
            if self.is_bus_high()? {
                return Ok(());
            }
            delay.delay_us(2);
        }
        Err(OneWireError::BusNotHigh)
    }

    /// Sends a reset pulse, then returns true if a device is present
    pub fn reset(&mut self, delay: &mut impl DelayUs<u16>) -> OneWireResult<bool, E> {
        self.wait_for_high(delay)?;

        self.set_bus_low()?;
        delay.delay_us(480); // Maxim recommended wait time

        self.release_bus()?;
        delay.delay_us(70); // Maxim recommended wait time

        let device_present = self.is_bus_low()?;

        delay.delay_us(410); // Maxim recommended wait time
        Ok(device_present)
    }

    pub fn read_bit(&mut self, delay: &mut impl DelayUs<u16>) -> OneWireResult<bool, E> {
        self.set_bus_low()?;
        delay.delay_us(6); // Maxim recommended wait time

        self.release_bus()?;
        delay.delay_us(9); // Maxim recommended wait time

        let bit_value = self.is_bus_high()?;
        delay.delay_us(55); // Maxim recommended wait time
        Ok(bit_value)
    }

    pub fn read_byte(&mut self, delay: &mut impl DelayUs<u16>) -> OneWireResult<u8, E> {
        let mut output: u8 = 0;
        for _ in 0..8 {
            output >>= 1;
            if self.read_bit(delay)? {
                output |= 0x80;
            }
        }
        Ok(output)
    }
    pub fn read_bytes(
        &mut self,
        output: &mut [u8],
        delay: &mut impl DelayUs<u16>,
    ) -> OneWireResult<(), E> {
        for i in 0..output.len() {
            output[i] = self.read_byte(delay)?;
        }
        Ok(())
    }

    pub fn write_1_bit(&mut self, delay: &mut impl DelayUs<u16>) -> OneWireResult<(), E> {
        self.set_bus_low()?;
        delay.delay_us(6); // Maxim recommended wait time

        self.release_bus()?;
        delay.delay_us(64); // Maxim recommended wait time
        Ok(())
    }

    pub fn write_0_bit(&mut self, delay: &mut impl DelayUs<u16>) -> OneWireResult<(), E> {
        self.set_bus_low()?;
        delay.delay_us(60); // Maxim recommended wait time

        self.release_bus()?;
        delay.delay_us(10); // Maxim recommended wait time
        Ok(())
    }

    pub fn write_bit(
        &mut self,
        value: bool,
        delay: &mut impl DelayUs<u16>,
    ) -> OneWireResult<(), E> {
        if value {
            self.write_1_bit(delay)
        } else {
            self.write_0_bit(delay)
        }
    }

    pub fn write_byte(
        &mut self,
        mut value: u8,
        delay: &mut impl DelayUs<u16>,
    ) -> OneWireResult<(), E> {
        for _ in 0..8 {
            self.write_bit(value & 0x01 == 0x01, delay)?;
            value >>= 1;
        }
        Ok(())
    }

    pub fn write_bytes(
        &mut self,
        bytes: &[u8],
        delay: &mut impl DelayUs<u16>,
    ) -> OneWireResult<(), E> {
        for i in 0..bytes.len() {
            self.write_byte(bytes[i], delay)?;
        }
        Ok(())
    }

    /// Address a specific device. All others will wait for a reset pulse.
    /// This should only be called after a reset, and should be immediately followed by another command
    pub fn match_address(
        &mut self,
        address: &Address,
        delay: &mut impl DelayUs<u16>,
    ) -> OneWireResult<(), E> {
        self.write_byte(commands::MATCH_ROM, delay)?;
        self.write_bytes(&address.0.to_le_bytes(), delay)?;
        Ok(())
    }

    /// Address all devices on the bus simultaneously.
    /// This should only be called after a reset, and should be immediately followed by another command
    pub fn skip_address(&mut self, delay: &mut impl DelayUs<u16>) -> OneWireResult<(), E> {
        self.write_byte(commands::SKIP_ROM, delay)?;
        Ok(())
    }

    /// Sends a reset, followed with either a SKIP_ROM or MATCH_ROM (with an address), and then the supplied command
    /// This should be followed by any reading/writing, if needed by the command used
    pub fn send_command(
        &mut self,
        command: u8,
        address: Option<&Address>,
        delay: &mut impl DelayUs<u16>,
    ) -> OneWireResult<(), E> {
        self.reset(delay)?;
        if let Some(address) = address {
            self.match_address(address, delay)?;
        } else {
            self.skip_address(delay)?;
        }
        self.write_byte(command, delay)?;
        Ok(())
    }

    /// Returns an iterator that iterates over all device addresses on the bus
    /// They can be filtered to only alarming devices if needed
    /// There is no requirement to immediately finish iterating all devices, but if devices are
    /// added / removed / change alarm state, the search may return an error or fail to find a device
    /// Device addresses will always be returned in the same order (lowest to highest, Little Endian)
    pub fn devices<'a, 'b, D>(
        &'a mut self,
        only_alarming: bool,
        delay: &'b mut D,
    ) -> DeviceSearch<'a, 'b, T, D>
    where
        D: DelayUs<u16>,
    {
        DeviceSearch {
            onewire: self,
            delay,
            state: None,
            finished: false,
            only_alarming,
        }
    }

    /// Search for device addresses on the bus
    /// They can be filtered to only alarming devices if needed
    /// Start the first search with a search_state of `None`, then use the returned state for subsequent searches
    /// There is no time limit for continuing a search, but if devices are
    /// added / removed / change alarm state, the search may return an error or fail to find a device
    /// Device addresses will always be returned in the same order (lowest to highest, Little Endian)
    pub fn device_search(
        &mut self,
        search_state: Option<&SearchState>,
        only_alarming: bool,
        delay: &mut impl DelayUs<u16>,
    ) -> OneWireResult<Option<(Address, SearchState)>, E> {
        if let Some(search_state) = search_state {
            if search_state.discrepancies == 0 {
                return Ok(None);
            }
        }

        if !self.reset(delay)? {
            return Ok(None);
        }
        if only_alarming {
            self.write_byte(commands::SEARCH_ALARM, delay)?;
        } else {
            self.write_byte(commands::SEARCH_NORMAL, delay)?;
        }

        let mut last_discrepancy_index: u8 = 0;
        let mut address;
        let mut discrepancies;
        let continue_start_bit;

        if let Some(search_state) = search_state {
            // follow up to the last discrepancy
            for bit_index in 0..search_state.last_discrepancy_index {
                let _false_bit = !self.read_bit(delay)?;
                let _true_bit = !self.read_bit(delay)?;
                let was_discrepancy_bit =
                    (search_state.discrepancies & (1_u64 << (bit_index as u64))) != 0;
                if was_discrepancy_bit {
                    last_discrepancy_index = bit_index;
                }
                let previous_chosen_bit =
                    (search_state.address & (1_u64 << (bit_index as u64))) != 0;

                // choose the same as last time
                self.write_bit(previous_chosen_bit, delay)?;
            }
            address = search_state.address;
            // This is the discrepancy bit. False is always chosen to start, so choose true this time
            {
                let false_bit = !self.read_bit(delay)?;
                let true_bit = !self.read_bit(delay)?;
                if !(false_bit && true_bit) {
                    // A different response was received than last search
                    return Err(OneWireError::UnexpectedResponse);
                }
                let address_mask = 1_u64 << (search_state.last_discrepancy_index as u64);
                address |= address_mask;
                self.write_bit(true, delay)?;
            }

            //keep all discrepancies except the last one
            discrepancies = search_state.discrepancies
                & !(1_u64 << (search_state.last_discrepancy_index as u64));
            continue_start_bit = search_state.last_discrepancy_index + 1;
        } else {
            address = 0;
            discrepancies = 0;
            continue_start_bit = 0;
        }
        for bit_index in continue_start_bit..64 {
            let false_bit = !self.read_bit(delay)?;
            let true_bit = !self.read_bit(delay)?;
            let chosen_bit = match (false_bit, true_bit) {
                (false, false) => {
                    // No devices responded to the search request
                    return Err(OneWireError::UnexpectedResponse);
                }
                (false, true) => {
                    // All remaining devices have the true bit set
                    true
                }
                (true, false) => {
                    // All remaining devices have the false bit set
                    false
                }
                (true, true) => {
                    // Discrepancy, multiple values reported
                    // choosing the lower value here
                    discrepancies |= 1_u64 << (bit_index as u64);
                    last_discrepancy_index = bit_index;
                    false
                }
            };
            let address_mask = 1_u64 << (bit_index as u64);
            if chosen_bit {
                address |= address_mask;
            } else {
                address &= !address_mask;
            }
            self.write_bit(chosen_bit, delay)?;
        }
        crc::check_crc8(&address.to_le_bytes())?;
        Ok(Some((
            Address(address),
            SearchState {
                address,
                discrepancies,
                last_discrepancy_index,
            },
        )))
    }
}

pub struct DeviceSearch<'a, 'b, T, D> {
    onewire: &'a mut OneWire<T>,
    delay: &'b mut D,
    state: Option<SearchState>,
    finished: bool,
    only_alarming: bool,
}

impl<'a, 'b, T, E, D> Iterator for DeviceSearch<'a, 'b, T, D>
where
    T: InputPin<Error = E>,
    T: OutputPin<Error = E>,
    D: DelayUs<u16>,
{
    type Item = OneWireResult<Address, E>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let result =
            self.onewire
                .device_search(self.state.as_ref(), self.only_alarming, self.delay);
        match result {
            Ok(Some((address, search_state))) => {
                self.state = Some(search_state);
                Some(Ok(address))
            }
            Ok(None) => {
                self.state = None;
                self.finished = true;
                None
            }
            Err(err) => {
                self.state = None;
                self.finished = true;
                Some(Err(err))
            }
        }
    }
}
