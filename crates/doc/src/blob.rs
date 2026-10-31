use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// A content address: the SHA-256 of an imported file (PDF, image, audio). The library stores
/// each blob once under this name, however many scores use it. Serialised as 64 hex digits.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct BlobRef(pub [u8; 32]);

impl BlobRef {
    /// Parses 64 hex digits (either case).
    pub fn of_hex(hex: &str) -> Result<BlobRef, String> {
        let bad = || {
            let shown: String = hex.chars().take(80).collect();
            format!("`{shown}` is not a blob id (64 hex digits)")
        };
        if hex.len() != 64 || !hex.is_ascii() {
            return Err(bad());
        }
        let mut out = [0u8; 32];
        for (i, b) in out.iter_mut().enumerate() {
            let pair = hex.get(i * 2..i * 2 + 2).ok_or_else(bad)?;
            *b = u8::from_str_radix(pair, 16).map_err(|_| bad())?;
        }
        Ok(BlobRef(out))
    }

    pub fn to_hex(self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }
}

impl fmt::Display for BlobRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for BlobRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BlobRef({})",
            self.to_hex().get(..12).unwrap_or_default()
        )
    }
}

impl FromStr for BlobRef {
    type Err = String;
    fn from_str(s: &str) -> Result<BlobRef, String> {
        BlobRef::of_hex(s)
    }
}

impl TryFrom<String> for BlobRef {
    type Error = String;
    fn try_from(s: String) -> Result<BlobRef, String> {
        BlobRef::of_hex(&s)
    }
}

impl From<BlobRef> for String {
    fn from(b: BlobRef) -> String {
        b.to_hex()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trip_and_rejects() {
        let h = "0123456789abcdef".repeat(4);
        let b = BlobRef::of_hex(&h).unwrap();
        assert_eq!(b.to_hex(), h);
        assert_eq!(BlobRef::of_hex(&h.to_uppercase()).unwrap(), b);
        assert!(BlobRef::of_hex(&h[..62]).is_err());
        assert!(BlobRef::of_hex(&"zz".repeat(32)).is_err());
        assert!(BlobRef::of_hex(&format!("{}é", &h[..62])).is_err());
    }
}
