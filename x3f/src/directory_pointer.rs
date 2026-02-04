use core::fmt;

use crate::X3FError;
use crate::debug_helper::TruncatedBytes;

/// # Structure
///
/// | Offset | Length | Item | Notes |
/// | --- | --- | --- | --- |
/// | 0 | 4 | Offset of start of directory section from start of file, in bytes. |  |
pub struct DirectoryPointerRef<'a> {
    bytes: &'a [u8],
}

impl fmt::Debug for DirectoryPointerRef<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("DirectoryPointerRef")
            .field("bytes", &TruncatedBytes(self.bytes))
            .finish()
    }
}

impl<'a> DirectoryPointerRef<'a> {
    pub const LENGTH: usize = 4;

    /// # Errors
    ///
    /// Returns `X3FError::TooShort` if the input is less than 4 bytes.
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
    pub fn offset(&self) -> &'a [u8] {
        &self.bytes[0..4]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const DIRECTORY_POINTER_SIZE: usize = 4;

    proptest! {
        #[test]
        fn directory_pointer_ref_returns_correct_slices(bytes in prop::collection::vec(any::<u8>(), DIRECTORY_POINTER_SIZE..=DIRECTORY_POINTER_SIZE)) {
            let ptr = DirectoryPointerRef { bytes: &bytes };

            prop_assert_eq!(ptr.as_bytes(), &bytes[..]);
            prop_assert_eq!(ptr.offset(), &bytes[0..4]);
        }
    }
}
