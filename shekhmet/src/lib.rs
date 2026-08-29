#![cfg_attr(not(test), no_std)]

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Shekhmet;

impl Shekhmet {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}
