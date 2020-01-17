# One-Wire Bus
[![Build Status](https://travis-ci.org/fuchsnj/one-wire-bus.svg?branch=master)](https://travis-ci.org/fuchsnj/one-wire-bus)
[![crates.io](https://img.shields.io/crates/v/one-wire-bus.svg)](https://crates.io/crates/one-wire-bus)
[![API](https://docs.rs/one-wire-bus/badge.svg)](https://docs.rs/one-wire-bus)

A Rust implementation of the [1-Wire](https://en.wikipedia.org/wiki/1-Wire) protocol for [embedded-hal](https://github.com/rust-embedded/embedded-hal)


## Quick Start
These examples omit error handling to keep them short. You should check all
results and handle them appropriately.

The 1-wire bus requires a single digital pin that is configured as an
open-drain output (it's either open, or connected to ground), and the bus
should have a ~5K Ohm pull-up resistor connected. How you obtain this pin from your
specific device is up the the embedded-hal implementation for that device, but it must
implement both `InputPin` and `OutputPin` 

```rust
use embedded_hal::blocking::delay::DelayUs;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use core::fmt::{Debug, Write};
use one_wire_bus::OneWire;

fn find_devices<P, E>(
    delay: &mut impl DelayUs<u16>,
    tx: &mut impl Write,
    one_wire_pin: P,
)
    where
        P: OutputPin<Error=E> + InputPin<Error=E>,
        E: Debug
{
    let mut one_wire_bus = OneWire::new(one_wire_pin).unwrap();
    for device_address in one_wire_bus.devices(false, delay) {
        // The search could fail at any time, so check each result. The iterator automatically
        // ends after an error.
        let device_address = device_address.unwrap();

        // The family code can be used to identify the type of device
        // If supported, another crate can be used to interact with that device at the given address
        writeln!(tx, "Found device at address {:?} with family code: {:#x?}",
                 device_address, device_address.family_code()).unwrap();
    }
}
```

Example Output
```
Found device at address E800000B1FCD1028 with family code: 0x28
Found device at address 70000008AC851628 with family code: 0x28
Found device at address 0B00000B20687E28 with family code: 0x28
Found device at address 5700000B2015FF28 with family code: 0x28
```
