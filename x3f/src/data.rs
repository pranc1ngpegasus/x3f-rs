use core::fmt;

use crate::X3FError;
use crate::debug_helper::TruncatedBytes;

/// # Data Subsection Types
///
/// | Type of Entry | Contents | Notes |
/// | --- | --- | --- |
/// | `"PROP"` | Property list. | List of pairs of strings. Each pair is a name and its corresponding value. |
/// | `"IMAG"` | Image data | Image data. Has a header indicating dimensions, pixel type, compression, amount of processing done. |
/// | `"IMA2"` | Image data | Image data. Readers should treat this the same as IMAG. Writers should use this for image sections that contain processed-for-preview data in other than uncompressed RGB24 pixel format. |
/// | `"CAMF"` | Camera metadata | Structure is undocumented; expose raw bytes. |
#[derive(Debug)]
pub enum SectionData<'a> {
    Prop(Prop<'a>),
    Image(Image<'a>),
    Ima2(Image<'a>),
    Camf(Camf<'a>),
}

/// # Structure
///
/// | Offset | Length | Item | Notes |
/// | --- | --- | --- | --- |
/// | 0 | 4 | Section identifier | Should be `"SECp"` |
/// | 4 | 4 | Property list format version | Should be 2.0 for now. |
/// | 8 | 4 | Number of property entries |  |
/// | 12 | 4 | Character format for all entries in this table. | 0 = CHAR16 Unicode. |
/// | 16 | 4 | RESERVED |  |
/// | 20 | 4 | Total length of name/value data in characters. |
pub struct Prop<'a> {
    bytes: &'a [u8],
}

impl fmt::Debug for Prop<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("Prop")
            .field("bytes", &TruncatedBytes(self.bytes))
            .finish()
    }
}

impl<'a> Prop<'a> {
    pub const LENGTH: usize = 24;

    /// Creates a new `Prop` from the given byte slice.
    ///
    /// # Errors
    ///
    /// Returns [`X3FError::TooShort`] if `bytes.len() < Self::LENGTH`.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, X3FError> {
        if bytes.len() < Self::LENGTH {
            return Err(X3FError::TooShort);
        }

        Ok(Self { bytes })
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
    pub fn property_list_format_version(&self) -> &'a [u8] {
        &self.bytes[4..8]
    }

    #[must_use]
    pub fn number_of_property_entries(&self) -> &'a [u8] {
        &self.bytes[8..12]
    }

    #[must_use]
    pub fn character_format(&self) -> &'a [u8] {
        &self.bytes[12..16]
    }

    #[must_use]
    pub fn reserved(&self) -> &'a [u8] {
        &self.bytes[16..20]
    }

    #[must_use]
    pub fn total_length_of_name_value_data(&self) -> &'a [u8] {
        &self.bytes[20..24]
    }
}

/// # Structure
///
/// | Offset | Length | Item | Notes |
/// | --- | --- | --- | --- |
/// | 0 | 4 | Section identifier | Should be `"SECp"` |
/// | 4 | 4 | Image format version | Should be 2.0 for now. |
/// | 8 | 4 | Type of image data | 2 = processed for preview (others RESERVED) |
/// | 12 | 4 | Data format | 3 = uncompressed 24-bit 8/8/8 RGB, 11 = Huffman-encoded DPCM 8/8/8 RGB, 18 = JPEG-compressed 8/8/8 RGB (others RESERVED) |
/// | 16 | 4 | Image columns | Image width / row size in pixels |
/// | 20 | 4 | Image rows | Image height in pixels |
/// | 24 | 4 | Row size in bytes | Will always be a multiple of 4 (32-bit aligned). A value of zero here means that rows are variable-length (as in Huffman data). |
pub struct Image<'a> {
    bytes: &'a [u8],
}

impl fmt::Debug for Image<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("Image")
            .field("bytes", &TruncatedBytes(self.bytes))
            .finish()
    }
}

impl<'a> Image<'a> {
    pub const LENGTH: usize = 28;

    /// Creates a new `Image` from the given byte slice.
    ///
    /// # Errors
    ///
    /// Returns [`X3FError::TooShort`] if `bytes.len() < Self::LENGTH`.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, X3FError> {
        if bytes.len() < Self::LENGTH {
            return Err(X3FError::TooShort);
        }

        Ok(Self { bytes })
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
    pub fn image_format_version(&self) -> &'a [u8] {
        &self.bytes[4..8]
    }

    #[must_use]
    pub fn type_of_image_data(&self) -> &'a [u8] {
        &self.bytes[8..12]
    }

    #[must_use]
    pub fn data_format(&self) -> &'a [u8] {
        &self.bytes[12..16]
    }

    #[must_use]
    pub fn image_columns(&self) -> &'a [u8] {
        &self.bytes[16..20]
    }

    #[must_use]
    pub fn image_rows(&self) -> &'a [u8] {
        &self.bytes[20..24]
    }

    #[must_use]
    pub fn row_size_in_bytes(&self) -> &'a [u8] {
        &self.bytes[24..28]
    }
}

/// Raw CAMF section data.
///
/// The CAMF structure is not documented in the public X3F spec, so we only
/// expose the raw bytes for now.
pub struct Camf<'a> {
    bytes: &'a [u8],
}

impl fmt::Debug for Camf<'_> {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        f.debug_struct("Camf")
            .field("bytes", &TruncatedBytes(self.bytes))
            .finish()
    }
}

impl<'a> Camf<'a> {
    pub const LENGTH: usize = 4;

    /// Creates a new `Camf` from the given byte slice.
    ///
    /// # Errors
    ///
    /// Returns [`X3FError::TooShort`] if `bytes.len() < Self::LENGTH`.
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, X3FError> {
        if bytes.len() < Self::LENGTH {
            return Err(X3FError::TooShort);
        }

        Ok(Self { bytes })
    }

    #[must_use]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    #[must_use]
    pub fn section_identifier(&self) -> &'a [u8] {
        &self.bytes[0..4]
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn prop_from_bytes_rejects_short_input() {
        let bytes = std::vec![0u8; Prop::LENGTH - 1];
        let err = Prop::from_bytes(&bytes).unwrap_err();
        match err {
            X3FError::TooShort => {},
            other => panic!("expected TooShort, got {other:?}"),
        }
    }

    #[test]
    fn image_from_bytes_rejects_short_input() {
        let bytes = std::vec![0u8; Image::LENGTH - 1];
        let err = Image::from_bytes(&bytes).unwrap_err();
        match err {
            X3FError::TooShort => {},
            other => panic!("expected TooShort, got {other:?}"),
        }
    }

    #[test]
    fn camf_from_bytes_rejects_short_input() {
        let bytes = std::vec![0u8; Camf::LENGTH - 1];
        let err = Camf::from_bytes(&bytes).unwrap_err();
        match err {
            X3FError::TooShort => {},
            other => panic!("expected TooShort, got {other:?}"),
        }
    }
}
