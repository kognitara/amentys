#![cfg_attr(not(test), no_std)]

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Douat;

impl Douat {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
