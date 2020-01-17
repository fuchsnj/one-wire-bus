use crate::error::{OneWireError, OneWireResult};

/// Calculates the crc8 of the input data.
pub fn crc8(data: &[u8]) -> u8 {
    let mut crc = 0;
    for byte in data {
        let mut byte = *byte;
        for _ in 0..8 {
            let x = (byte ^ crc) & 0x01;
            crc >>= 1;
            if x != 0 {
                crc ^= 0x8C;
            }
            byte >>= 1;
        }
    }
    crc
}

/// Checks to see if data (including the crc byte) passes the crc check.
///
/// A nice property of this crc8 algorithm is that if you include the crc value in the data
/// it will always return 0, so it's not needed to separate the data from the crc value
pub fn check_crc8<E>(data: &[u8]) -> OneWireResult<(), E> {
    if crc8(data) == 0 {
        Ok(())
    } else {
        Err(OneWireError::CrcMismatch)
    }
}

#[cfg(test)]
mod test {
    use crate::crc::crc8;

    #[test]
    fn test_crc8() {
        assert_eq!(crc8(&[99, 1, 75, 70, 127, 255, 13, 16]), 21);
        assert_eq!(crc8(&[99, 1, 75, 70, 127, 255, 13, 16, 21]), 0);

        assert_eq!(crc8(&[97, 1, 75, 70, 127, 255, 15, 16]), 2);
        assert_eq!(crc8(&[97, 1, 75, 70, 127, 255, 15, 16, 2]), 0);

        assert_eq!(crc8(&[95, 1, 75, 70, 127, 255, 1, 16]), 155);
        assert_eq!(crc8(&[95, 1, 75, 70, 127, 255, 1, 16, 155]), 0);
    }
}
