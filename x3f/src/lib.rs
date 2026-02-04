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

        let extended_header = if header.file_format_version() >= b"2.1" {
            Some(ExtendedHeaderRef::from_bytes(
                &bytes[HeaderRef::LENGTH..HeaderRef::LENGTH + ExtendedHeaderRef::LENGTH],
            )?)
        } else {
            None
        };

        let directory_pointer =
            DirectoryPointerRef::from_bytes(&bytes[bytes.len() - DirectoryPointerRef::LENGTH..])?;

        let offset =
            u32::from_le_bytes(directory_pointer.offset().try_into().unwrap_or([0u8; 4])) as usize;
        let directory = DirectoryRef::from_bytes(&bytes[offset..])?;

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

        if offset + length > self.bytes.len() {
            return None;
        }

        let data_bytes = &self.bytes[offset..offset + length];

        match entry_type {
            b"PROP" => Some(SectionData::Prop(Prop::from_bytes(data_bytes))),
            b"IMAG" => Some(SectionData::Image(Image::from_bytes(data_bytes))),
            b"IMA2" => Some(SectionData::Ima2(Image::from_bytes(data_bytes))),
            _ => None,
        }
    }
}
