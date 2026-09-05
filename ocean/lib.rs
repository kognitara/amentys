//! CoreOcean — working set adressé par Noun.
//!
//! Remplace `ocean/src/lib.rs`.
//! DiskOcean (NVMe / `ra`) vient après : même trait, autre backend.
//!
//! `put` / `get` tiennent en RAM no_std. Un reboot QEMU vide CoreOcean ;
//! c'est voulu. La persistance est DiskOcean.

#![cfg_attr(not(test), no_std)]

use noun::Noun;

pub const SLOT_BYTES: usize = 256;
pub const MAX_SLOTS: usize = 64;

struct Slot {
    noun: Noun,
    len: u16,
    data: [u8; SLOT_BYTES],
}

/// Surface de l'océan : working set.
pub struct CoreOcean {
    slots: heapless::Vec<Slot, MAX_SLOTS>,
}

impl CoreOcean {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            slots: heapless::Vec::new(),
        }
    }

    /// Adresse le contenu. Même bytes ⇒ même Noun.
    pub fn put(&mut self, bytes: &[u8]) -> Result<Noun, &'static str> {
        if bytes.len() > SLOT_BYTES {
            return Err("blob exceeds SLOT_BYTES");
        }
        let noun = Noun::of(bytes);
        if let Some(existing) = self.find(noun) {
            return Ok(existing);
        }
        let mut data = [0u8; SLOT_BYTES];
        data[..bytes.len()].copy_from_slice(bytes);
        self.slots
            .push(Slot {
                noun: noun.clone(),
                len: bytes.len() as u16,
                data,
            })
            .map_err(|_| "CoreOcean full")?;
        Ok(noun)
    }

    #[must_use]
    pub fn get(&self, noun: Noun) -> Option<&[u8]> {
        self.slots
            .iter()
            .find(|s| s.noun == noun)
            .map(|s| &s.data[..s.len as usize])
    }

    #[must_use]
    pub fn contains(&self, noun: Noun) -> bool {
        self.get(noun).is_some()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }

    fn find(&self, noun: Noun) -> Option<Noun> {
        self.slots.iter().find(|s| s.noun == noun).map(|s| s.noun.clone())
    }
}

impl Default for CoreOcean {
    fn default() -> Self {
        Self::new()
    }
}

impl CoreOcean {
    pub fn push(&mut self, noun: Noun) {
        // compat : un Noun sans blob n'entre pas. no-op volontaire.
        let _ = noun;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_get_stable() {
        let mut o = CoreOcean::new();
        let a = o.put(b"hello").unwrap();
        let b = o.put(b"hello").unwrap();
        assert_eq!(a, b);
        assert_eq!(o.get(a), Some(&b"hello"[..]));
        assert_eq!(o.len(), 1);
        assert_eq!(a, Noun::of(b"hello"));
    }

    #[test]
    fn unknown_is_none() {
        let o = CoreOcean::new();
        assert!(o.get(Noun::of(b"nope")).is_none());
    }
}
