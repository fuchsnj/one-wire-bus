use core::fmt;

/// The result of a 1-Wire operation, parameterized by the GPIO error type.
pub type OneWireResult<T, E> = Result<T, OneWireError<E>>;

/// An error from a 1-Wire operation or device driver.
#[derive(Debug, Copy, Clone)]
pub enum OneWireError<E> {
    /// The pull-up resistor did not bring the released bus high in time.
    BusNotHigh,

    /// The GPIO operation failed.
    PinError(E),

    /// A command received an unexpected bus response.
    ///
    /// This can happen when devices join or leave the bus during a search.
    UnexpectedResponse,

    /// The address has a different family code than the driver expects.
    FamilyCodeMismatch,
    /// The ROM or data failed its CRC check.
    CrcMismatch,
    /// The device did not finish its operation in time.
    Timeout,
}

impl<E: fmt::Debug> fmt::Display for OneWireError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BusNotHigh => f.write_str("the 1-Wire bus did not rise after release"),
            Self::PinError(error) => write!(f, "1-Wire GPIO error: {:?}", error),
            Self::UnexpectedResponse => f.write_str("unexpected 1-Wire response"),
            Self::FamilyCodeMismatch => f.write_str("unexpected 1-Wire device family code"),
            Self::CrcMismatch => f.write_str("1-Wire CRC mismatch"),
            Self::Timeout => f.write_str("1-Wire operation timed out"),
        }
    }
}

/// Available when the `std` feature is enabled.
///
/// ```
/// use core::convert::Infallible;
/// use one_wire_bus::OneWireError;
///
/// let error: Box<dyn std::error::Error> =
///     OneWireError::<Infallible>::Timeout.into();
/// assert_eq!(error.to_string(), "1-Wire operation timed out");
/// ```
#[cfg(feature = "std")]
impl<E: fmt::Debug> std::error::Error for OneWireError<E> {}
