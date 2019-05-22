use std::{error::Error, fmt};

#[derive(Debug, PartialEq)]
enum ValidationError {
    Truncated,
    OverAllocated { declared: usize, actual: usize },
    BadPadding,
    UnknownTypeCode { code: u8 },
    Overflow,
}

impl Error for ValidationError {}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Truncated => write!(f, "Buffer has been truncated"),
            OverAllocated => write!(f, "Buffer has been overallocated"),
            BadPadding => write!(f, "Unexpected bytes at beginning of buffer"),
            UnknownTypeCode => write!(f, "Unrecognized type code"),
            Overflow => write!(f, "Can't represent claimed bounds"),
        }
    }
}

fn validate(buffer: &[u8]) -> Result<(), ValidationError> {
    if buffer.len() < 4 {
        return Err(ValidationError::Truncated);
    }

    if buffer[0] != 0 || buffer[1] != 0 {
        return Err(ValidationError::BadPadding);
    }

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::{validate, ValidationError};

    /// Checks that `idx_validate` correctly flags bad padding.
    #[test]
    fn test_validate_bad_padding() {
        #[rustfmt::skip]
        let mut data: [u8; 12] = [
            0x00, 0x00, 0x08, 0x01,
            0x00, 0x00, 0x00, 0x04,
            0x01, 0x02, 0x03, 0x04,
        ];

        for padding in 0x0001u16..=0xffffu16 {
            assert_eq!(validate(&data), Err(ValidationError::BadPadding));
        }
    }

    /// Checks that `idx_validate` will safely reject arrays that are longer than
    /// a `size_t`.
    #[test]
    fn validate_overflow_int16() {
        // Note that we can't actually allocate an array longer than the maximum
        // allowed by a 64 bit `size_t`.  This doesn't matter as the overflow will
        // happen before we hit the comparison.
        // Bounds chosen to multiply to exactly two to the power of 64 minus 16,
        // over 2 meaning that the structure is one byte two big.
        #[rustfmt::skip]
        let data: [u8; 16] = [
            0x00, 0x00, 0x0B, 0x03,
            0x00, 0x00, 0x05, 0x29,
            0x03, 0x54, 0x4a, 0xb8,
            0x07, 0x73, 0x62, 0xf1,
        ];

        assert_eq!(validate(&data), Err(ValidationError::Overflow));
    }

    /// Checks that `idx_validate` will safely reject arrays that are longer than
    /// a `size_t`.
    #[test]
    fn test_validate_overflow_uint8() {
        // Note that we can't actually allocate an array longer than the maximum
        // allowed by a 64 bit `size_t`.  This doesn't matter as the overflow will
        // happen before we hit the comparison.
        // Bounds chosen to multiply to exactly two to the power of 64 minus 16,
        // meaning that the structure is one byte two big.
        #[rustfmt::skip]
        let data: [u8; 16] = [
            0x00, 0x00, 0x08, 0x03,
            0x00, 0x00, 0x05, 0x29,
            0x06, 0xa8, 0x95, 0x70,
            0x07, 0x73, 0x62, 0xf1,
        ];

        assert_eq!(validate(&data), Err(ValidationError::Overflow));
    }

    /// Checks that `idx_validate` correctly rejects structures that have not been
    /// allocated enough space to contain their bounds.
    #[test]
    fn test_validate_too_short_for_bounds() {
        #[rustfmt::skip]
        let data: [u8; 11] = [
            0x00, 0x00, 0x08, 0x02,
            0x00, 0x00, 0x00, 0x03,
            0x00, 0x00, 0x00,
        ];

        assert_eq!(validate(&data), Err(ValidationError::Truncated));
    }

    /// Checks that `idx_validate` will correctly reject structures that have not
    /// been allocated enough space for all of their data.
    #[test]
    fn test_validate_too_short_for_data_1d() {
        #[rustfmt::skip]
        let data: [u8; 9] = [
            0x00, 0x00, 0x08, 0x01,
            0x00, 0x00, 0x00, 0x03,
            0x01,
        ];

        assert_eq!(validate(&data), Err(ValidationError::Truncated));
    }

    /// Checks that `idx_validate` will correctly reject structures that have not
    /// been allocated enough space for all of their data.
    #[test]
    fn test_validate_too_short_for_data_3d() {
        // Data is one byte shorter than declared in the header..
        #[rustfmt::skip]
        let data: [u8; 31] = [
            0x00, 0x00, 0x0b, 0x03,
            0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x02,
            0x00, 0x00, 0x00, 0x02,
            0x01, 0x02, 0x03, 0x04,
            0x05, 0x06, 0x07, 0x08,
            0x09, 0x0a, 0x0b, 0x0c,
            0x0e, 0x0f, 0xa0,
        ];

        assert_eq!(validate(&data), Err(ValidationError::Truncated));
    }

    /// Checks that `idx_validate` will correctly reject structures that have not
    /// been allocated enough space to fit the value containing the number of
    /// dimensions.
    #[test]
    fn test_validate_too_short_for_bounds_header() {
        #[rustfmt::skip]
        let data: [u8; 3] = [
            0x00, 0x00, 0x08,
        ];

        assert_eq!(validate(&data), Err(ValidationError::Truncated));
    }

    /// Checks that `idx_validate` will correctly reject structures that have not
    /// been allocated enough space to fit the type code.
    #[test]
    fn test_validate_too_short_for_type_header() {
        #[rustfmt::skip]
        let data: [u8; 2] = [
            0x00, 0x00,
        ];

        assert_eq!(validate(&data), Err(ValidationError::Truncated));
    }

    /// Checks that `idx_validate` will accept a valid zero-dimensional uint8 array.
    #[test]
    fn test_validate_uint8_0d() {
        #[rustfmt::skip]
        let data: [u8; 5] = [
            0x00, 0x00, 0x08, 0x00,
            0xfe,
        ];

        assert_eq!(validate(&data), Ok(()));
    }

    /// Checks that `idx_validate` will accept a normal 2d uint8 array.
    #[test]
    fn test_validate_uint8_2d() {
        // A 3x3 identity matrix.
        #[rustfmt::skip]
        let data: [u8; 21] = [
            0x00, 0x00, 0x08, 0x02,
            0x00, 0x00, 0x00, 0x03,
            0x00, 0x00, 0x00, 0x03,
            0x01, 0x00, 0x00,
            0x00, 0x01, 0x00,
            0x00, 0x00, 0x01,
        ];

        assert_eq!(validate(&data), Ok(()));
    }
}
