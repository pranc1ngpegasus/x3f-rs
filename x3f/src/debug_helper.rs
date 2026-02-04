const TRUNCATE_THRESHOLD: usize = 16;

pub struct TruncatedBytes<'a>(pub &'a [u8]);

impl core::fmt::Debug for TruncatedBytes<'_> {
    fn fmt(
        &self,
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        let bytes = self.0;
        if bytes.len() <= TRUNCATE_THRESHOLD {
            write!(f, "{bytes:?}")
        } else {
            write!(f, "[")?;
            for (i, byte) in bytes.iter().take(TRUNCATE_THRESHOLD).enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{byte}")?;
            }
            write!(f, ", ...] ({} bytes)", bytes.len())
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;
    use std::format;

    #[test]
    fn truncated_bytes_short() {
        let bytes = [1, 2, 3, 4, 5];
        let debug_str = format!("{:?}", TruncatedBytes(&bytes));
        assert_eq!(debug_str, "[1, 2, 3, 4, 5]");
    }

    #[test]
    fn truncated_bytes_exact_threshold() {
        let bytes = [0u8; 16];
        let debug_str = format!("{:?}", TruncatedBytes(&bytes));
        assert_eq!(
            debug_str,
            "[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]"
        );
    }

    #[test]
    fn truncated_bytes_long() {
        let bytes = [0u8; 100];
        let debug_str = format!("{:?}", TruncatedBytes(&bytes));
        assert_eq!(
            debug_str,
            "[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, ...] (100 bytes)"
        );
    }
}
