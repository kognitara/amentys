use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

const STACK_SIZE: usize = 4096 * 5; // 20 Ko

// L'index IST utilisé pour le rattrapage du Double Fault
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
#[repr(align(16))]
struct InterruptStack([u8; STACK_SIZE]);
// On utilise une structure simple pour s'assurer que la mémoire existe
static mut INT_STACK: InterruptStack = InterruptStack([0; STACK_SIZE]);

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();

        // Calcul de l'adresse brute en pointant sur le tableau interne (.0)
        let stack_start_ptr = unsafe {
            // SAFETY: L'adresse brute est garantie valide car elle pointe sur une structure statique alignée.
            &raw const INT_STACK.0 as u64
         };
        let stack_end = stack_start_ptr + STACK_SIZE as u64;

        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = VirtAddr::new_truncate(stack_end);
        tss.privilege_stack_table[0] = VirtAddr::new_truncate(stack_end);

        tss
    };
}

pub struct Selectors {
    pub kernel_code: SegmentSelector,
    pub kernel_data: SegmentSelector,
    pub user_code_32: SegmentSelector,
    pub user_data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub tss_selector: SegmentSelector,
}
lazy_static! {
    pub static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();

        let kernel_code = gdt.append(Descriptor::kernel_code_segment());
        let kernel_data = gdt.append(Descriptor::kernel_data_segment());

        // On capture le premier segment pour servir de base matérielle à SYSRET
        let user_code_32 = gdt.append(Descriptor::user_code_segment());
        let user_data = gdt.append(Descriptor::user_data_segment());
        let user_code = gdt.append(Descriptor::user_code_segment());

        let tss_selector = gdt.append(Descriptor::tss_segment(&TSS));

        (gdt, Selectors {
            kernel_code,
            kernel_data,
            user_code_32, // <-- Exporte-le ici
            user_data,
            user_code,
            tss_selector,
        })
    };
}
pub fn init() {
    GDT.0.load();
    // SAFETY: Chargement des sélecteurs d'index de segments et de la tâche TSS dans le CPU.
    unsafe {
        use x86_64::instructions::segmentation::{CS, DS, ES, SS, Segment};
        use x86_64::instructions::tables::load_tss;

        CS::set_reg(GDT.1.kernel_code);
        DS::set_reg(GDT.1.kernel_data);
        ES::set_reg(GDT.1.kernel_data);
        SS::set_reg(GDT.1.kernel_data);

        load_tss(GDT.1.tss_selector);
    }
}
