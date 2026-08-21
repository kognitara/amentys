use limine::memmap::{Entry, MEMMAP_USABLE};
use x86_64::{
    PhysAddr, VirtAddr,
    structures::paging::{FrameAllocator, OffsetPageTable, PageTable, Size4KiB},
};

/// Allocator of physical frames based on the Limine memory map.
///
/// # Fields
/// - `memory_map`: Reference to the Limine memory map entries.
/// - `next_free_frame`: Index of the next free frame to allocate.
#[allow(dead_code)]
pub struct BootInfoFrameAllocator {
    memory_map: &'static [&'static Entry],
    next_free_frame: usize,
}

impl BootInfoFrameAllocator {
    /// Crée un nouvel allocateur à partir de la carte mémoire de Limine.
    #[must_use]
    pub const fn new(memory_map: &'static [&'static Entry]) -> Self {
        Self {
            memory_map,
            next_free_frame: 0,
        }
    }
}
/// Safety: The `BootInfoFrameAllocator` is safe to use as a frame allocator because it only allocates frames from usable memory regions as defined by the Limine memory map. The `allocate_frame` method ensures that it does not allocate frames from non-usable regions, and it maintains an internal index to track the next free frame, preventing double allocation of the same frame.    
unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<x86_64::structures::paging::PhysFrame<Size4KiB>> {
        let mut current_frame_index = 0;

        for region in self.memory_map {
            // we only care about usable memory regions
            if region.type_ != MEMMAP_USABLE {
                continue;
            }

            let region_base = region.base;
            let region_length = region.length;
            let frames_in_region = region_length / 4096; // 4 KiB per frame

            // only the next free frame index falls within this memory region
            if self.next_free_frame < current_frame_index + (frames_in_region as usize) {
                // On calcule l'offset exact de la frame dans cette région
                let frame_offset = (self.next_free_frame - current_frame_index) as u64;
                let phys_addr = PhysAddr::new(region_base + (frame_offset * 4096));

                self.next_free_frame += 1;

                return Some(x86_64::structures::paging::PhysFrame::containing_address(
                    phys_addr,
                ));
            }
            // Otherwise, we advance our virtual index by the size of this region
            current_frame_index += frames_in_region as usize;
        }
        // here we have exhausted all usable memory regions, so we return None
        None
    }
}
/// Initialise l'`OffsetPageTable` à partir de l'adresse de base HHDM.
///
/// # Safety
///
/// this function is unsafe because the caller must ensure that the provided `physical_memory_offset` is valid and corresponds to the actual physical memory mapping.
///
/// The function retrieves the active level 4 page table and constructs an `OffsetPageTable` using this offset.
///
#[must_use]
pub unsafe fn init_paging(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
    // SAFETY: Get the active level 4 page table via the physical memory offset.
    let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };

    unsafe {
        // SAFETY: The caller must ensure that the provided `physical_memory_offset` is valid and corresponds to the actual physical memory mapping.
        OffsetPageTable::new(level_4_table, physical_memory_offset)
    }
}

/// Get the active level 4 page table.
///
/// # Arguments
/// - `physical_memory_offset`: The offset to the physical memory mapping (HHDM).
///
/// ```no_run
/// let physical_memory_offset = VirtAddr::new(0xFFFF800000000000);
/// let level_4_table = unsafe { active_level_4_table(physical_memory_offset) };
/// ```
unsafe fn active_level_4_table(physical_memory_offset: VirtAddr) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    let (level_4_table_frame, _) = Cr3::read();
    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    // SAFETY: The constructed virtual address of the level 4 page table is guaranteed to be valid by the bootloader.
    unsafe { &mut *page_table_ptr }
}
