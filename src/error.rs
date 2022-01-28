use core::fmt::Debug;

pub type OneWireResult<T, E> = Result<T, OneWireError<E>>;

#[derive(Debug, Copy, Clone)]
pub enum OneWireError<E> {
    /// The Bus was expected to be pulled high by a ~5K ohm pull-up resistor, but it wasn't
    BusNotHigh,

    PinError(E),

    /// An unexpected response was received from a command. This generally happens when a new sensor is added
    /// or removed from the bus during a command, such as a device search.
    UnexpectedResponse,

    FamilyCodeMismatch,
    CrcMismatch,
    Timeout,
}

#[cfg(feature = "std")]
impl<E: Debug> std::error::Error for OneWireError<E> {}

#[cfg(feature = "std")]
impl<E: Debug> core::fmt::Display for OneWireError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
