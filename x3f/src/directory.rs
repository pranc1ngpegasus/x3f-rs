use core::fmt;

use crate::X3FError;
use crate::debug_helper::TruncatedBytes;

/// # Structure
///
/// | Offset | Length | Item | Notes |
/// | --- | --- | --- | --- |
/// | 0 | 4 | Section Identifier | Contains `"SECd"` |
/// | 4 | 4 | Section Version | Section version. Should be 2.0 for now. |
/// | 8 | 4 | Number of directory entries. | Note: Original spec incorrectly shows offset 4. |
pub struct DirectoryRef<'a> {
    bytes: &'a [u8],
}

impl fmt::Debug for DirectoryRef<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("DirectoryRef")
            .field("bytes", &TruncatedBytes(self.bytes))
            .finish()
    }
}

impl<'a> DirectoryRef<'a> {
    /// # Errors
    ///
    /// Returns `X3FError::TooShort` if the input is less than 12 bytes.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, X3FError> {
        if bytes.len() < 12 {
            return Err(X3FError::TooShort);
        }

        Ok(Self { bytes: &bytes[0..] })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    #[must_use]
    pub fn section_identifier(&self) -> &'a [u8] {
        &self.bytes[0..4]
    }

    #[must_use]
    pub fn section_version(&self) -> &'a [u8] {
        &self.bytes[4..8]
    }

    #[must_use]
    pub fn entry_count(&self) -> &'a [u8] {
        &self.bytes[8..12]
    }

    #[must_use]
    pub fn entries(&self) -> DirectoryEntriesIter<'a> {
        DirectoryEntriesIter {
            bytes: &self.bytes[12..],
            pos: 0,
        }
    }
}

pub struct DirectoryEntriesIter<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl fmt::Debug for DirectoryEntriesIter<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("DirectoryEntriesIter")
            .field("bytes", &TruncatedBytes(self.bytes))
            .field("pos", &self.pos)
            .finish()
    }
}

impl<'a> Iterator for DirectoryEntriesIter<'a> {
    type Item = DirectoryEntryRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + 12 <= self.bytes.len() {
            let entry = DirectoryEntryRef {
                bytes: &self.bytes[self.pos..self.pos + 12],
            };
            self.pos += 12;
            Some(entry)
        } else {
            None
        }
    }
}

/// # Structure
///
/// | Offset | Length | Item | Notes |
/// | --- | --- | --- | --- |
/// | 0 | 4 | Offset from start of file to start of entry's data, in bytes. | Offset must be a multiple of 4, so that the data starts on a 32-bit boundary. |
/// | 4 | 4 | Length of entry's data, in bytes. |  |
/// | 8 | 4 | Type of entry. | See below for a list of valid types. |
pub struct DirectoryEntryRef<'a> {
    bytes: &'a [u8],
}

impl fmt::Debug for DirectoryEntryRef<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("DirectoryEntryRef")
            .field("bytes", &TruncatedBytes(self.bytes))
            .finish()
    }
}

impl<'a> DirectoryEntryRef<'a> {
    #[must_use]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    #[must_use]
    pub fn data_offset(&self) -> &'a [u8] {
        &self.bytes[0..4]
    }

    #[must_use]
    pub fn data_length(&self) -> &'a [u8] {
        &self.bytes[4..8]
    }

    #[must_use]
    pub fn entry_type(&self) -> &'a [u8] {
        &self.bytes[8..12]
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use proptest::prelude::*;
    use std::vec::Vec;

    const DIRECTORY_HEADER_SIZE: usize = 12;
    const DIRECTORY_ENTRY_SIZE: usize = 12;

    prop_compose! {
        fn arbitrary_directory_bytes(max_entries: usize)
            (entry_count in 0..=max_entries)
            (
                header in prop::array::uniform12(any::<u8>()),
                entries in prop::collection::vec(
                    prop::array::uniform12(any::<u8>()),
                    entry_count..=entry_count
                )
            ) -> Vec<u8> {
            let mut bytes = header.to_vec();
            for entry in entries {
                bytes.extend_from_slice(&entry);
            }
            bytes
        }
    }

    proptest! {
        #[test]
        fn directory_ref_returns_correct_slices(bytes in arbitrary_directory_bytes(10)) {
            let dir = DirectoryRef { bytes: &bytes };

            prop_assert_eq!(dir.as_bytes(), &bytes[..]);
            prop_assert_eq!(dir.section_identifier(), &bytes[0..4]);
            prop_assert_eq!(dir.section_version(), &bytes[4..8]);
            prop_assert_eq!(dir.entry_count(), &bytes[8..12]);
        }

        #[test]
        fn entries_iter_returns_correct_count(bytes in arbitrary_directory_bytes(10)) {
            let dir = DirectoryRef { bytes: &bytes };
            let expected_count = (bytes.len() - DIRECTORY_HEADER_SIZE) / DIRECTORY_ENTRY_SIZE;

            prop_assert_eq!(dir.entries().count(), expected_count);
        }

        #[test]
        fn entries_iter_returns_correct_slices(bytes in arbitrary_directory_bytes(10)) {
            let dir = DirectoryRef { bytes: &bytes };

            for (i, entry) in dir.entries().enumerate() {
                let start = DIRECTORY_HEADER_SIZE + i * DIRECTORY_ENTRY_SIZE;
                let end = start + DIRECTORY_ENTRY_SIZE;
                prop_assert_eq!(entry.as_bytes(), &bytes[start..end]);
            }
        }

        #[test]
        fn directory_entry_ref_returns_correct_slices(bytes in prop::array::uniform12(any::<u8>())) {
            let entry = DirectoryEntryRef { bytes: &bytes };

            prop_assert_eq!(entry.as_bytes(), &bytes[..]);
            prop_assert_eq!(entry.data_offset(), &bytes[0..4]);
            prop_assert_eq!(entry.data_length(), &bytes[4..8]);
            prop_assert_eq!(entry.entry_type(), &bytes[8..12]);
        }
    }

    #[test]
    fn entries_iter_handles_partial_entry() {
        // 12 bytes header + 6 bytes (partial entry) = 18 bytes
        let bytes = [0u8; 18];
        let dir = DirectoryRef { bytes: &bytes };

        // Partial entry should be ignored
        assert_eq!(dir.entries().count(), 0);
    }

    #[test]
    fn entries_iter_handles_exact_boundary() {
        // 12 bytes header + 12 bytes (1 entry) = 24 bytes
        let bytes = [0u8; 24];
        let dir = DirectoryRef { bytes: &bytes };

        assert_eq!(dir.entries().count(), 1);
    }
}
