# One-Wire Bus

[![CI](https://github.com/fuchsnj/one-wire-bus/actions/workflows/ci.yml/badge.svg?branch=master)](https://github.com/fuchsnj/one-wire-bus/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/one-wire-bus.svg)](https://crates.io/crates/one-wire-bus)
[![API](https://docs.rs/one-wire-bus/badge.svg)](https://docs.rs/one-wire-bus)

A blocking, `no_std` implementation of the 1-Wire protocol for
[embedded-hal 1.0](https://github.com/rust-embedded/embedded-hal).

## Quick start

Connect the data line to an open-drain GPIO with an external pull-up resistor
(typically 4.7 kΩ). The pin must implement both `InputPin` and `OutputPin` in
that mode. For example, with `stm32f4xx-hal`, split the GPIO port and configure
an available pin with `into_open_drain_output()`. Supply a board delay that
implements `embedded_hal::delay::DelayNs`.

```rust,no_run
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::{InputPin, OutputPin};
use one_wire_bus::{Address, OneWire, OneWireResult};

fn find_devices<P: InputPin + OutputPin>(
    pin: P,
    delay: &mut impl DelayNs,
    mut report: impl FnMut(Address),
) -> OneWireResult<(), P::Error> {
    let mut bus = OneWire::new(pin)?;
    // Pass true instead to search only for devices with an active alarm.
    for address in bus.devices(false, delay) {
        report(address?);
    }
    Ok(())
}
```

Pass the configured pin and delay to `find_devices`. The callback receives
each ROM address and can call `Address::family_code()` to identify the device
type. The iterator stops after the first error.

If searches return `UnexpectedResponse`, check that the GPIO can read the line
while released and that the delay is calibrated to the configured CPU clock.
The bus is bit-banged, so GPIO overhead and interrupts affect the sample time.

## Upgrading from 0.1

Replace HAL 0.2 delay implementations with HAL 1.0's `DelayNs` and use the GPIO
traits from `embedded_hal::digital`. Sampling with `is_bus_high` or
`is_bus_low` now requires a mutable bus handle, matching HAL 1.0's `InputPin`.

The optional `std` feature adds `std::error::Error` support for
`OneWireError`. It is disabled by default.
