#![cfg_attr(not(test), no_std)]

use core::{arch::asm, fmt::Write};
/// Exits the program with the given exit code using the `exit` syscall.
/// # Safety
/// This function is unsafe because it uses inline assembly and directly interacts with the system call interface.
pub fn sys_weigh_heart(code: usize) -> ! {
    unsafe {
        // Safety: We are using inline assembly to perform a syscall.
        // The parameters are set according to the syscall convention for Linux x86_64.
        asm!(
            "syscall",
            in("rax") 60,
            in("rdi") code,
            options(noreturn)
        );
    }
}

pub fn sys_write(text: &str) {
    unsafe {
        // Safety: We are using inline assembly to perform a syscall.
        // The parameters are set according to the syscall convention for Linux x86_64.
        asm!(
            "syscall",
            inout("rax") 1 => _, // RAX prend '1' en entrée, et sa valeur de retour est ignorée (_)
            in("rdi") 1,
            in("rsi") text.as_ptr(),
            in("rdx") text.len(),
            out("rcx") _,
            out("r11") _,
        );
    }
}

#[derive(Debug, Default)]
pub struct Anubis {
    pub code: usize,
}

impl Anubis {
    /// Creates a new [`Anubis`].
    ///
    /// # Parameters
    /// - `code`: The exit code to be used when the program exits
    #[must_use]
    pub const fn new(code: usize) -> Self {
        Self { code }
    }

    /// Calls the provided closure to set the exit code for the [`Anubis`] instance.
    ///
    /// # Parameters
    /// - `f`: A closure that returns the exit code to be set for the [`Anubis`] instance   
    pub fn call(&mut self, f: impl Fn(Scribe) -> usize) {
        self.code = f(Scribe::new());
    }

    pub fn judge<F>(f: F) -> !
    where
        F: FnOnce(&mut Scribe) -> usize,
    {
        let mut scribe = Scribe::new();
        let weight = f(&mut scribe);
        sys_weigh_heart(weight);
    }
}

#[derive(Debug, Default)]
pub struct Scribe;

impl Scribe {
    /// Creates a new [`Scribe`].
    ///
    /// # Parameters
    /// - `code`: The exit code to be used when the program exits
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }

    /// Writes the provided string to the standard output.
    /// # Parameters
    /// - `text`: The string to be written to the standard output
    pub fn write_raw(&self, text: &str) {
        sys_write(text);
    }
}

impl Write for Scribe {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_raw(s);
        Ok(())
    }
}
