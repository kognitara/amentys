// Autorise les gestionnaires d'exceptions matérielles non invoqués manuellement
use crate::gdt::DOUBLE_FAULT_IST_INDEX;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT\n{stack_frame:#?}");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: u64,
) {
    panic!("EXCEPTION: GENERAL PROTECTION FAULT (Code {error_code})\n{stack_frame:#?}");
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    panic!(
        "EXCEPTION: PAGE FAULT\nAdresse accédée: {:?}\nCode d'erreur: {error_code:?}\n{stack_frame:#?}",
        Cr2::read(),
    );
}
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();

        // 1. Les handlers "normaux"
        idt.general_protection_fault.set_handler_fn(general_protection_fault_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);

        // 2. Le handler critique (Double Fault) lié à l'IST
        // SAFETY: L'index IST fourni correspond à une pile valide configurée dans le TSS.
        unsafe {
            idt.double_fault.set_handler_fn(double_fault_handler)
                .set_stack_index(DOUBLE_FAULT_IST_INDEX);
        }

        idt
    };
}

pub fn init() {
    // Charge l'IDT dans le processeur via l'instruction `lidt`
    IDT.load();
}
