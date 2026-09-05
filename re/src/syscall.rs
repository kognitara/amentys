use crate::gdt::GDT;
use core::arch::naked_asm;
use ra::TERMINAL;
use x86_64::registers::model_specific::{Efer, EferFlags, LStar, SFMask, Star};
use x86_64::registers::rflags::RFlags;
/// init the syscall handler by setting up the necessary MSRs and flags.
///
/// # Panics
/// This function will panic if it fails to write to the MSR STAR register.
#[allow(clippy::panic, clippy::expect_used)]
pub fn init() {
    unsafe {
        // Safety: activate the syscall extension in the EFER register
        Efer::update(|efer| efer.insert(EferFlags::SYSTEM_CALL_EXTENSIONS));

        Star::write(
            GDT.1.user_code,
            GDT.1.user_data,
            GDT.1.kernel_code,
            GDT.1.kernel_data,
        )
        .expect("Erreur lors de l'écriture dans le registre MSR STAR");

        LStar::write(x86_64::VirtAddr::new(
            syscall_handler_wrapper as *const () as u64,
        ));

        SFMask::write(RFlags::INTERRUPT_FLAG);
    }
}

#[unsafe(naked)]
pub extern "C" fn syscall_handler_wrapper() {
    naked_asm!(
        "push rcx",
        "push r11",
        // Alignement mathématique strict de la pile sur 16 octets
        "push rbp",
        "mov rbp, rsp",
        "and rsp, -16",
        "mov rcx, rdx",
        "mov rdx, rsi",
        "mov rsi, rdi",
        "mov rdi, rax",
        "call handle_syscall",
        // Restauration de la pile
        "mov rsp, rbp",
        "pop rbp",
        "pop r11",
        "pop rcx",
        "sysretq"
    );
}

/// Handle the syscall based on the syscall number and arguments.
///
/// # Panics
/// This function will panic if it encounters an invalid text length or an unexpected exit code.
///
#[allow(clippy::panic)]
#[unsafe(no_mangle)]
pub extern "C" fn handle_syscall(syscall_no: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    match syscall_no {
        1 => {
            let ptr = arg2 as *const u8;

            // Validation propre sans clippy warning
            let Ok(len) = usize::try_from(arg3) else {
                panic!("Invalid text length in syscall");
            };

            let slice = unsafe {
                // Safety: We trust that the pointer and length provided by the syscall are valid.
                core::slice::from_raw_parts(ptr, len)
            };

            if let Some(ref mut terminal) = *TERMINAL.lock() {
                terminal.process(slice);
            }
            0
        }
        60 => {
            if arg1 == 0 {
                // Gèle définitivement la machine virtuelle en toute sécurité
                x86_64::instructions::interrupts::disable();
                loop {
                    x86_64::instructions::hlt();
                }
            }
            panic!("Amentys Kernel: Maât a échoué (exit code non nul)");
        }
        _ => u64::MAX,
    }
}
