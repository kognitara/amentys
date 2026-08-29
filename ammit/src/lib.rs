#![cfg_attr(not(test), no_std)]

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Ammit;

impl Ammit {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
