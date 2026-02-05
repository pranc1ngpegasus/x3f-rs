#![no_std]

mod data;
mod debug_helper;
mod directory;
mod directory_pointer;
mod header;

pub use crate::data::{Image, Prop, SectionData};
pub use crate::directory::{DirectoryEntriesIter, DirectoryEntryRef, DirectoryRef};
pub use crate::directory_pointer::DirectoryPointerRef;
pub use crate::header::{ExtendedHeaderRef, HeaderRef};

use core::fmt;

use crate::debug_helper::TruncatedBytes;

/// # Structure
///
/// | Section | Notes |
/// | --- | --- |
/// | Header |  |
/// | Extended Header | header extension |
/// | Data |  |
/// | Directory | Directory of subsections in the data section. |
/// | Directory Pointer | Offset from the start of the file to the start of the directory section, in bytes. |
pub struct X3F<'a> {
    bytes: &'a [u8],
    header: HeaderRef<'a>,
    extended_header: Option<ExtendedHeaderRef<'a>>,
    directory_pointer: DirectoryPointerRef<'a>,
    directory: DirectoryRef<'a>,
}

impl fmt::Debug for X3F<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("X3F")
            .field("bytes", &TruncatedBytes(self.bytes))
            .field("header", &self.header)
            .field("extended_header", &self.extended_header)
            .field("directory_pointer", &self.directory_pointer)
            .field("directory", &self.directory)
            .finish()
    }
}

#[derive(Debug)]
pub enum X3FError {
    TooShort,
    InvalidFileType,
    OutOfBounds,
}

impl<'a> X3F<'a> {
    /// # Errors
    ///
    /// Returns `X3FError::TooShort` if the input is too small to contain a valid X3F structure.
    /// Returns `X3FError::InvalidFileType` if the file type identifier is not `"FOVb"`.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, X3FError> {
        if bytes.len() < HeaderRef::LENGTH + DirectoryPointerRef::LENGTH {
            return Err(X3FError::TooShort);
        }

        let header = HeaderRef::from_bytes(&bytes[0..HeaderRef::LENGTH])?;
        if header.file_type_identifier() != b"FOVb" {
            return Err(X3FError::InvalidFileType);
        }

        let extended_header =
            if u32::from_le_bytes(header.file_format_version().try_into().unwrap_or([0u8; 4]))
                > 0x2000
            {
                let range = HeaderRef::LENGTH..HeaderRef::LENGTH + ExtendedHeaderRef::LENGTH;
                let extended_bytes = bytes.get(range).ok_or(X3FError::TooShort)?;
                Some(ExtendedHeaderRef::from_bytes(extended_bytes)?)
            } else {
                None
            };

        let directory_pointer =
            DirectoryPointerRef::from_bytes(&bytes[bytes.len() - DirectoryPointerRef::LENGTH..])?;

        let offset = u32::from_le_bytes(
            directory_pointer
                .offset()
                .try_into()
                .map_err(|_| X3FError::TooShort)?,
        ) as usize;
        let directory_bytes = bytes.get(offset..).ok_or(X3FError::OutOfBounds)?;
        let directory = DirectoryRef::from_bytes(directory_bytes)?;

        Ok(Self {
            bytes,
            header,
            extended_header,
            directory_pointer,
            directory,
        })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    #[must_use]
    pub fn header(&self) -> &HeaderRef<'a> {
        &self.header
    }

    #[must_use]
    pub fn extended_header(&self) -> Option<&ExtendedHeaderRef<'a>> {
        self.extended_header.as_ref()
    }

    #[must_use]
    pub fn directory_pointer(&self) -> &DirectoryPointerRef<'a> {
        &self.directory_pointer
    }

    #[must_use]
    pub fn directory(&self) -> &DirectoryRef<'a> {
        &self.directory
    }

    #[must_use]
    pub fn section_data(
        &self,
        entry: &DirectoryEntryRef<'a>,
    ) -> Option<SectionData<'a>> {
        let offset = u32::from_le_bytes(entry.data_offset().try_into().ok()?) as usize;
        let length = u32::from_le_bytes(entry.data_length().try_into().ok()?) as usize;
        let entry_type = entry.entry_type();

        let end = offset.checked_add(length)?;
        let data_bytes = self.bytes.get(offset..end)?;

        match entry_type {
            b"PROP" => Prop::from_bytes(data_bytes).ok().map(SectionData::Prop),
            b"IMAG" => Image::from_bytes(data_bytes).ok().map(SectionData::Image),
            b"IMA2" => Image::from_bytes(data_bytes).ok().map(SectionData::Ima2),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::vec::Vec;

    fn make_header(file_format_version: [u8; 4]) -> [u8; HeaderRef::LENGTH] {
        let mut header = [0u8; HeaderRef::LENGTH];
        header[0..4].copy_from_slice(b"FOVb");
        header[4..8].copy_from_slice(&file_format_version);
        header
    }

    #[test]
    fn from_bytes_rejects_out_of_bounds_directory_offset() {
        let mut bytes = Vec::new();
        // Use version <= 0x2000 so no extended header is required
        bytes.extend_from_slice(&make_header([0u8; 4]));
        bytes.extend_from_slice(&1000u32.to_le_bytes());

        let err = X3F::from_bytes(&bytes).unwrap_err();
        match err {
            X3FError::OutOfBounds => {},
            other => panic!("expected OutOfBounds, got {other:?}"),
        }
    }

    #[test]
    fn from_bytes_rejects_missing_extended_header() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&make_header(*b"2.1\0"));
        bytes.extend_from_slice(&0u32.to_le_bytes());

        let err = X3F::from_bytes(&bytes).unwrap_err();
        match err {
            X3FError::TooShort => {},
            other => panic!("expected TooShort, got {other:?}"),
        }
    }

    #[test]
    fn section_data_returns_none_for_out_of_bounds_entry() {
        let mut bytes = Vec::new();
        // Use version <= 0x2000 so no extended header is required
        bytes.extend_from_slice(&make_header([0u8; 4]));

        let directory_offset = HeaderRef::LENGTH as u32;
        let directory_start = bytes.len();

        // Directory header (12 bytes)
        bytes.extend_from_slice(b"SECd");
        bytes.extend_from_slice(b"2.0\0");
        bytes.extend_from_slice(&1u32.to_le_bytes());

        // Directory entry (12 bytes)
        bytes.extend_from_slice(&60u32.to_le_bytes());
        bytes.extend_from_slice(&20u32.to_le_bytes());
        bytes.extend_from_slice(b"PROP");

        let directory_len = bytes.len() - directory_start;
        let dir_ptr_pos = bytes.len();
        bytes.resize(dir_ptr_pos + DirectoryPointerRef::LENGTH, 0);
        bytes[dir_ptr_pos..dir_ptr_pos + 4].copy_from_slice(&directory_offset.to_le_bytes());

        assert_eq!(directory_len, 24);

        let x3f = X3F::from_bytes(&bytes).expect("valid X3F");
        let entry = x3f.directory().entries().next().expect("entry");
        assert!(x3f.section_data(&entry).is_none());
    }
}
