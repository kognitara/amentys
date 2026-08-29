#![cfg_attr(not(test), no_std)]

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Seth;

impl Seth {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod test {
    #[test]
    pub fn s() {
        assert!(true);
    }
}
