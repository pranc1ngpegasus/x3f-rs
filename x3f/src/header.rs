use core::fmt;

use crate::X3FError;
use crate::debug_helper::TruncatedBytes;

/// # Structure
///
/// ## Versions 2.1 - 2.2
///
/// | Offset | Length | Item | Notes |
/// | --- | --- | --- | --- |
/// | 0 | 4 | File type identifier | Contains `"FOVb"` |
/// | 4 | 4 | File format version | File format version (2.1–2.2) |
/// | 8 | 16 | Unique identifier | Guaranteed unique per image (not UUID compatible) |
/// | 24 | 4 | Mark bits | Used to denote marked subsets |
/// | 28 | 4 | Image columns | Width of unrotated image |
/// | 32 | 4 | Image rows | Height of unrotated image |
/// | 36 | 4 | Rotation | Clockwise rotation: 0, 90, 180, 270 |
/// | 40 | 32 | White balance label string | ASCIIZ white balance label |
/// | 72 | 32 | Extended data types | 32 × 8-bit type identifiers |
/// | 104 | 128 | Extended data | 32 × 32-bit extended data values |
///
/// ## Versions 2.0 and older
///
/// | Offset | Length | Item | Notes |
/// | --- | --- | --- | --- |
/// | 0 | 4 | File type identifier | Contains `"FOVb"` |
/// | 4 | 4 | File format version | File format version (≤ 2.0) |
/// | 8 | 16 | Unique identifier | Guaranteed unique per image (not UUID compatible) |
/// | 24 | 4 | Mark bits | Used to denote marked subsets |
/// | 28 | 4 | Image columns | Width of unrotated image |
/// | 32 | 4 | Image rows | Height of unrotated image |
/// | 36 | 4 | Rotation | Clockwise rotation: 0, 90, 180, 270 |
pub struct HeaderRef<'a> {
    bytes: &'a [u8],
}

impl fmt::Debug for HeaderRef<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("HeaderRef")
            .field("bytes", &TruncatedBytes(self.bytes))
            .finish()
    }
}

impl<'a> HeaderRef<'a> {
    pub const LENGTH: usize = 40;

    /// # Errors
    ///
    /// Returns `X3FError::TooShort` if the input is less than 40 bytes.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, X3FError> {
        if bytes.len() < Self::LENGTH {
            return Err(X3FError::TooShort);
        }

        Ok(Self {
            bytes: &bytes[0..Self::LENGTH],
        })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    #[must_use]
    pub fn file_type_identifier(&self) -> &'a [u8] {
        &self.bytes[0..4]
    }

    #[must_use]
    pub fn file_format_version(&self) -> &'a [u8] {
        &self.bytes[4..8]
    }

    #[must_use]
    pub fn unique_identifier(&self) -> &'a [u8] {
        &self.bytes[8..24]
    }

    #[must_use]
    pub fn mark_bits(&self) -> &'a [u8] {
        &self.bytes[24..28]
    }

    #[must_use]
    pub fn image_columns(&self) -> &'a [u8] {
        &self.bytes[28..32]
    }

    #[must_use]
    pub fn image_rows(&self) -> &'a [u8] {
        &self.bytes[32..36]
    }

    #[must_use]
    pub fn rotation(&self) -> &'a [u8] {
        &self.bytes[36..40]
    }
}

/// Extended Header is an optional section that follows Header only in versions 2.1 - 2.2.
pub struct ExtendedHeaderRef<'a> {
    bytes: &'a [u8],
}

impl fmt::Debug for ExtendedHeaderRef<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("ExtendedHeaderRef")
            .field("bytes", &TruncatedBytes(self.bytes))
            .finish()
    }
}

impl<'a> ExtendedHeaderRef<'a> {
    pub const LENGTH: usize = 192;

    /// # Errors
    ///
    /// Returns `X3FError::TooShort` if the input is less than 192 bytes.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, X3FError> {
        if bytes.len() < Self::LENGTH {
            return Err(X3FError::TooShort);
        }

        Ok(Self {
            bytes: &bytes[0..Self::LENGTH],
        })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    #[must_use]
    pub fn white_balance_label_string(&self) -> &'a [u8] {
        &self.bytes[0..32]
    }

    #[must_use]
    pub fn extended_data_types(&self) -> &'a [u8] {
        &self.bytes[32..64]
    }

    #[must_use]
    pub fn extended_data(&self) -> &'a [u8] {
        &self.bytes[64..192]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const HEADER_SIZE: usize = 40;
    const EXTENDED_HEADER_SIZE: usize = 192;

    proptest! {
        #[test]
        fn header_ref_returns_correct_slices(bytes in prop::collection::vec(any::<u8>(), HEADER_SIZE..=HEADER_SIZE)) {
            let header = HeaderRef { bytes: &bytes };

            prop_assert_eq!(header.as_bytes(), &bytes[..]);
            prop_assert_eq!(header.file_type_identifier(), &bytes[0..4]);
            prop_assert_eq!(header.file_format_version(), &bytes[4..8]);
            prop_assert_eq!(header.unique_identifier(), &bytes[8..24]);
            prop_assert_eq!(header.mark_bits(), &bytes[24..28]);
            prop_assert_eq!(header.image_columns(), &bytes[28..32]);
            prop_assert_eq!(header.image_rows(), &bytes[32..36]);
            prop_assert_eq!(header.rotation(), &bytes[36..40]);
        }

        #[test]
        fn extended_header_ref_returns_correct_slices(bytes in prop::collection::vec(any::<u8>(), EXTENDED_HEADER_SIZE..=EXTENDED_HEADER_SIZE)) {
            let extended = ExtendedHeaderRef { bytes: &bytes };

            prop_assert_eq!(extended.as_bytes(), &bytes[..]);
            prop_assert_eq!(extended.white_balance_label_string(), &bytes[0..32]);
            prop_assert_eq!(extended.extended_data_types(), &bytes[32..64]);
            prop_assert_eq!(extended.extended_data(), &bytes[64..192]);
        }
    }
}
