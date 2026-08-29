#![cfg_attr(not(test), no_std)]

use noun::Noun;
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ocean {
    nodes: heapless::Vec<Noun, 256>,
}

impl Ocean {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: heapless::Vec::new(),
        }
    }

    pub fn push(&mut self, noun: Noun) {
        let _ = self.nodes.push(noun);
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}
