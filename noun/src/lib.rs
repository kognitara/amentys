#![cfg_attr(not(test), no_std)]

use core::clone::Clone;
use core::cmp::{Eq, Ord, PartialEq, PartialOrd};
use core::derive;
use core::fmt::Debug;

pub struct NounOcean {}

/// A Noun is a 32-byte identifier used in the Amentys system. It is represented as a fixed-size array of 32 bytes.
///
/// # Fields
/// * `hash` - A 32-byte array representing the hash of the noun.
#[repr(C)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Noun {
    hash: [u8; 32],
}

impl Noun {
    /// Creates a new [`Noun`].
    ///
    /// # Arguments
    /// * `hash` - A 32-byte array representing the hash of the noun.
    /// # Returns
    /// * A new instance of [`Noun`].
    /// # Example
    /// ```no_run
    /// use noun::Noun;
    /// let hash = [0u8; 32];  
    /// let noun = Noun::new(hash);
    /// ```
    #[must_use]
    pub const fn new(hash: [u8; 32]) -> Self {
        Self { hash }
    }
    #[must_use]
    pub const fn null() -> Self {
        Self { hash: [0u8; 32] }
    }
    /// Checks if this [`Noun`] is the null identifier (all zeros).
    /// # Returns
    /// * `true` if the noun is null, `false` otherwise.
    ///
    /// # Example
    /// ```no_run
    /// use noun::Noun;
    /// let null_noun = Noun::new([0u8; 32]);
    /// let non_null_noun = Noun::new([1u8; 32]);
    /// assert!(!non_null_noun.is_null());
    /// assert!(null_noun.is_null());
    /// ```
    #[must_use]
    pub const fn is_null(&self) -> bool {
        let mut i = 0;
        while i < self.hash.len() {
            if self.hash[i] != 0 {
                return false;
            }
            i += 1;
        }
        true
    }

    /// Creates a [`Noun`] from a byte slice. Fails if the size is not exactly 32 bytes.
    ///
    /// # Arguments
    /// * `bytes` - A byte slice that should be exactly 32 bytes long.
    /// # Returns
    /// * `Some(Noun)` if the slice is 32 bytes long, `None` if it is not.
    /// # Example
    /// ```no_run
    /// use noun::Noun;
    /// let valid_bytes = [1u8; 32];
    /// let noun = Noun::from_bytes(&valid_bytes).expect("Should succeed with 32 bytes");
    /// let invalid_bytes = [1u8; 31];
    /// assert!(Noun::from_bytes(&invalid_bytes).is_none());
    /// ```
    #[must_use]
    pub const fn from_bytes(bytes: &[u8]) -> core::option::Option<Self> {
        if bytes.len() != 32 {
            return core::option::Option::None;
        }
        let mut hash = [0u8; 32];
        hash.copy_from_slice(bytes);
        core::option::Option::Some(Self { hash })
    }

    /// Returns a reference to the internal 32-byte hash
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.hash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::assert;
    use core::assert_eq;
    #[test]
    fn test_noun_creation_and_bytes() {
        let hash = [1u8; 32];
        let noun = Noun::new(hash);
        assert_eq!(noun.as_bytes(), &hash);
        assert!(!noun.is_null());
    }

    #[test]
    fn test_noun_null() {
        let null_noun = Noun::null();
        assert!(null_noun.is_null());
    }

    #[test]
    fn test_noun_from_bytes() {
        let valid_bytes = [42u8; 32];
        let noun = Noun::from_bytes(&valid_bytes).expect("Should succeed with 32 bytes");
        assert_eq!(noun.as_bytes(), &valid_bytes);

        let invalid_bytes = [42u8; 31]; // Trop court
        assert!(Noun::from_bytes(&invalid_bytes).is_none());
    }
}
