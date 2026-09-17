use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // 1. Pile par défaut pour les interruptions Ring 3 -> Ring 0 (Crucial !)
        tss.privilege_stack_table[0] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut RSP0_STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            // Plus de bloc unsafe : la macro addr_of! est safe en Rust Nightly récent
            let stack_start = VirtAddr::from_ptr( core::ptr::addr_of!(RSP0_STACK));
            stack_start + STACK_SIZE as u64
        };

        // 2. Pile IST pour le Double Fault
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(core::ptr::addr_of!(STACK) );
            stack_start + STACK_SIZE as u64
        };

        tss
    };
}

lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        // Index 1: 0x08 (Kernel Code)
        let kernel_code = gdt.append(Descriptor::kernel_code_segment());

        // Index 2: 0x10 (Kernel Data)
        let kernel_data = gdt.append(Descriptor::kernel_data_segment());

        // Index 3: 0x18 (Dummy entry pour décaler et respecter tes offsets Ring 3)
        let _dummy = gdt.append(Descriptor::user_data_segment());

        // Index 4: 0x20 -> 0x23 avec RPL 3 (SS Utilisateur)
        let user_data = gdt.append(Descriptor::user_data_segment());

        // Index 5: 0x28 -> 0x2B avec RPL 3 (CS Utilisateur)
        let user_code = gdt.append(Descriptor::user_code_segment());

        // Index 6 & 7: 0x30 (TSS)
        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

        (
            gdt,
            Selectors {
                kernel_code,
                kernel_data,
                user_data,
                user_code,
                tss_selector,
            },
        )
    };
}

pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub tss_selector: SegmentSelector,
}

pub fn init() {
    // 1. Chargement de la nouvelle GDT
    GDT.0.load();

    // SAFETY: Configuration système bas niveau des registres de segments et du TSS
    unsafe {
        // 2. L'EXORCISME : On écrase les registres 0x30 de Limine avec nos propres segments
        CS::set_reg(GDT.1.kernel_code);
        SS::set_reg(GDT.1.kernel_data);
        DS::set_reg(GDT.1.kernel_data);
        ES::set_reg(GDT.1.kernel_data);

        // 3. Chargement du Task State Segment pour sécuriser l'interrupts.rs
        load_tss(GDT.1.tss_selector);
    }
}
