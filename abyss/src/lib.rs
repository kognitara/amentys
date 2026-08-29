#![cfg_attr(not(test), no_std)]

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Abyss;

impl Default for Abyss {
    fn default() -> Self {
        Self::new()
    }
}
impl Abyss {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}
