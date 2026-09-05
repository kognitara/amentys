#![cfg_attr(not(test), no_main, no_std)]
extern crate alloc;
use alloc::collections::BTreeMap;
use core::panic::PanicInfo;
use jinshu::{
    ocean::CoreOceanBuilder,
    router::{NounIndex, SemanticRouter},
    storage::StorageEngine,
};
use limine::request::HhdmRequest;
use noun::Noun;
use plan::{Plan, layer::Layers};
use prism::Prism;
use ra::fs::nvme::driver::init_all_nvme_devices;
use ra::{fs::nvme::allocator::Allocator, println};
use x86_64::instructions::hlt;
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();
use linked_list_allocator::LockedHeap;
// 1. On déclare l'allocateur global pour ce binaire
#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

// 2. On crée une fonction pour l'initialiser
pub fn init_heap() {
    const HEAP_SIZE: usize = 128 * 1024; // 128 Ko de RAM allouée à Maat

    // Un bloc de mémoire statique rempli de zéros
    static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

    // SAFETY: On donne ce bloc exclusif à notre allocateur
    unsafe {
        let heap_ptr = core::ptr::addr_of_mut!(HEAP).cast::<u8>();
        ALLOCATOR.lock().init(heap_ptr, HEAP_SIZE);
    }
}

/// Entry point of the kernel. This function is called by the bootloader after the kernel is loaded into memory.
///
/// # Safety
/// This function is marked as unsafe because it is the entry point of the kernel and is called by the bootloader.
///
/// It is the responsibility of the caller to ensure that the kernel is loaded correctly and that the bootloader has set up the environment correctly.
///
/// The function does not return, as it enters an infinite loop after executing the kernel code.
///
/// # Panics
///
/// This function may panic if the kernel encounters an unrecoverable error during execution.
///
/// In such cases, the panic handler will be invoked, which will print the panic message and enter an infinite loop.  
///
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    // 1. LA CARTE DE LA RÉALITÉ (HHDM)
    // On demande à Limine le décalage magique entre le physique et le virtuel

    let hhdm_offset = HHDM_REQUEST
        .response()
        .expect("Limine did not provide the HHDM offset!")
        .offset;

    // 2. ALLOCATION MÉMOIRE PHYSIQUE (Safe Zone)
    let mut allocator = Allocator::new(0x0100_0000, 0x0200_0000);

    // 3. HARDWARE : Réveil du NVMe
    // On passe le hhdm_offset au driver pour qu'il mappe ses files DMA correctement en virtuel
    let nvme_devices = init_all_nvme_devices(hhdm_offset, &mut allocator);
    let mut storage_engine = StorageEngine::new(1, &nvme_devices[0]);
    let disk_index = NounIndex::new();

    // 4. JINSHU : Instanciation du CoreOcean
    let mut core_ocean = BTreeMap::new();

    // On demande une page physique pour faire le pont NVMe -> RAM
    let buffer_phys = allocator
        .allocate_page()
        .expect("OOM: Not enough physical RAM for the DMA bridge");

    // MAGIE HHDM : On calcule la vraie adresse virtuelle utilisable par le CPU
    let buffer_virt = buffer_phys + hhdm_offset;

    // Hashs fondamentaux de l'OS (Terminal et Réseau)
    let tui_layer_noun = Noun::of(&[0x01; 32]);
    let network_layer_noun = Noun::of(&[0x02; 32]);

    // Aspiration dans la RAM partagée : Les données arriveront dans la RAM physique
    // via le SSD, mais notre CPU les lira via l'adresse virtuelle !
    CoreOceanBuilder::deep_load(
        &tui_layer_noun,
        &mut storage_engine,
        &disk_index,
        &mut core_ocean,
        buffer_phys,
        buffer_virt,
    )
    .expect("Failed to deep load TUI layer");

    CoreOceanBuilder::deep_load(
        &network_layer_noun,
        &mut storage_engine,
        &disk_index,
        &mut core_ocean,
        buffer_phys,
        buffer_virt,
    )
    .expect("Failed to deep load network layer");

    // 5. JINSHU (Le Routeur)
    let mut router = SemanticRouter::new(storage_engine, &core_ocean);

    // 6. LE PLAN : Forge du Terminal
    let terminal_root_noun = Noun::of(&[0xAA; 32]);
    let mut phoenix_layers = Layers::new(terminal_root_noun.clone());
    let terminal_plan = Plan::new(terminal_root_noun, &mut phoenix_layers, 0)
        .expect("Failed to create terminal plan");
    let sceau = plan::sceau::Sceau::birth(&terminal_plan, 1_000);
    match maat::law::weigh(&sceau, &terminal_plan) {
        plan::sceau::Verdict::Accept => {
            // 7. LE PRISME : Exécution de l'Application sans binaire
            let mut root_prism = Prism::new(terminal_plan);
            root_prism
                .run(&mut router)
                .expect("Failed to run the root prism");
            loop {
                hlt();
            }
        }
        plan::sceau::Verdict::Refuse(why) => panic!("maat refused: {why}"),
    }
}

#[cfg_attr(not(test), panic_handler)]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info.message());
    loop {
        hlt();
    }
}
