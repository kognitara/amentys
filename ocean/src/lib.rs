#![cfg_attr(not(test), no_std)]
use noun::Noun;

#[cfg(test)]
use zuu::zuu;

pub const SLOT_BYTES: usize = 256;
pub const MAX_SLOTS: usize = 64;

struct Slot {
    noun: Noun,
    len: u16,
    data: [u8; SLOT_BYTES],
}

/// Surface of ocean.
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
    /// Addresses the content. Same bytes ⇒ same `Noun`.
    ///
    /// # Panics
    ///
    /// Panics if the blob exceeds `SLOT_BYTES`.
    pub fn put(&mut self, bytes: &[u8]) -> Result<Noun, &'static str> {
        if bytes.len() > SLOT_BYTES {
            return Err("blob exceeds SLOT_BYTES");
        }
        let noun = Noun::of(bytes);
        if let Some(existing) = self.find(&noun) {
            return Ok(existing);
        }
        let mut data = [0u8; SLOT_BYTES];
        data[..bytes.len()].copy_from_slice(bytes);
        self.slots
            .push(Slot {
                noun: noun.clone(),
                len: u16::try_from(bytes.len()).expect("blob exceeds SLOT_BYTES"),
                data,
            })
            .map_err(|_| "CoreOcean full")?;
        Ok(noun)
    }
    /// Retrieves the content associated with the given `Noun`.
    #[must_use]
    pub fn get(&self, noun: &Noun) -> Option<&[u8]> {
        self.slots
            .iter()
            .find(|s| &s.noun == noun)
            .map(|s| &s.data[..s.len as usize])
    }
    /// Checks if the `CoreOcean` contains the given `Noun`.
    #[must_use]
    pub fn contains(&self, noun: &Noun) -> bool {
        self.get(noun).is_some()
    }
    /// Returns the number of `Nouns` stored in the `CoreOcean`.
    #[must_use]
    pub fn len(&self) -> usize {
        self.slots.len()
    }
    /// Checks if the `CoreOcean` is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
    /// Finds the `Noun` associated with the given `Noun`.
    #[must_use]
    fn find(&self, noun: &Noun) -> Option<Noun> {
        self.slots
            .iter()
            .find(|s| &s.noun == noun)
            .map(|s| s.noun.clone())
    }
}

impl Default for CoreOcean {
    fn default() -> Self {
        Self::new()
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
        assert!(
            zuu!([
                a.clone().eq(&b),
                o.get(&a.clone()).is_some(),
                o.get(&a.clone()).eq(&Some(&b"hello"[..])),
                o.len().eq(&1),
                a.eq(&Noun::of(b"hello")),
            ])
            .is_ok()
        );
    }

    #[test]
    fn unknown_is_none() {
        let o = CoreOcean::new();
        assert!(o.get(&Noun::of(b"nope")).is_none());
    }
}
