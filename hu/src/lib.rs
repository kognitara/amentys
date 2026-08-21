#![cfg_attr(not(test), no_main, no_std)]

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Hu;

impl Default for Hu {
    fn default() -> Self {
        Self::new()
    }
}

impl Hu {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}
