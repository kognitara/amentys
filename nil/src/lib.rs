#![cfg_attr(not(test), no_std)]

use noun::Noun;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Nil;

impl Nil {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    #[must_use]
    pub const fn is_null_noun(noun: &Noun) -> bool {
        noun.is_null()
    }
}
