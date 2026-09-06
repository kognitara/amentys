#![cfg_attr(not(test), no_std, no_main)]
use ra::println;

#[macro_export]
macro_rules! zuu {
    ($t:expr) => {{
        let mut z = $crate::Zuu::new();
        for x in $t {
            z.check("success", "failure", x);
        }
        z.end()
    }};
}

pub struct Zuu {
    pub failures: usize,
    pub success: usize,
}

impl Default for Zuu {
    fn default() -> Self {
        Self::new()
    }
}

impl Zuu {
    /// Create a new Zuu instance.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            failures: 0,
            success: 0,
        }
    }
    /// Check a condition and print a success or failure message.
    ///
    /// # Params
    /// - `s`: The success message to print.
    /// - `f`: The failure message to print.
    /// - `c`: The condition to check.
    pub fn check(&mut self, s: &str, f: &str, c: bool) -> &mut Self {
        if c {
            self.success += 1;
            println!("{s}");
        } else {
            self.failures += 1;
            println!("{f}");
        }
        self
    }
    /// Print the final results of the Zuu instance.
    ///
    /// # Returns
    /// - `Ok(success_count)`: If there were no failures.
    /// - `Err(failure_count)`: If there were failures.
    pub fn end(&mut self) -> Result<usize, usize> {
        println!(
            "failures: {}/{}",
            self.failures,
            self.failures + self.success
        );
        println!("success: {}/{}", self.success, self.failures + self.success);
        if self.failures >= 1 {
            Err(self.failures)
        } else {
            Ok(self.success)
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_end() {
        assert!(zuu!([true; 30]).is_ok());
    }
}
