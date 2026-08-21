/// Allocator for physical memory pages and disk nodes.
pub const LENGTH_PAGE: u64 = 4096;

/// Allocator for physical memory pages.
///
/// # Example
/// ```
/// let mut allocator = Allocator::new(0x1000, 0x10000);
/// let page = allocator.allocate_page().expect("Allocation failed");
/// assert_eq!(page % 4096, 0, "Page is not aligned!");
/// ```
/// # Fields
/// - `current_address`: The current address to allocate from.
/// - `limit`: The limit address for allocation.
/// # Note
/// This allocator does not clean the allocated memory. The cleaning is delegated to the kernel via virtual addresses.
pub struct Allocator {
    current_address: u64,
    limit: u64,
}

impl Allocator {
    /// Creates a new `Allocator` with the given start address and size.
    ///
    /// # Arguments
    /// * `start` - The starting address for allocation.
    /// * `size` - The size of the memory region to allocate from.
    /// # Returns
    /// A new instance of `Allocator`.
    ///
    /// # Example
    /// ```no_run
    /// use ra::fs::nvme::allocator::Allocator;
    /// let mut allocator = Allocator::new(0x1000, 0x10000);
    /// let page = allocator.allocate_page().expect("Allocation failed");
    /// assert_eq!(page % 4096, 0, "Page is not aligned!");
    /// ```
    #[must_use]
    pub const fn new(start: u64, size: u64) -> Self {
        Self {
            current_address: start,
            limit: start + size,
        }
    }
    /// Allocates a 4 KiB page that is aligned.
    /// # Returns
    /// An `Option` containing the aligned address of the allocated page, or `None` if the allocation exceeds the limit.
    ///
    /// # Example
    /// ```no_run
    /// use ra::fs::nvme::allocator::Allocator;
    /// let mut allocator = Allocator::new(0x1000, 0x10000);
    /// let page = allocator.allocate_page().expect("Allocation failed");
    /// assert_eq!(page % 4096, 0, "Page is not aligned!");
    /// ```
    #[must_use]
    pub const fn allocate_page(&mut self) -> core::option::Option<u64> {
        let aligned_address = (self.current_address + LENGTH_PAGE - 1) & !(LENGTH_PAGE - 1);
        let next_address = aligned_address + LENGTH_PAGE;

        if next_address > self.limit {
            return core::option::Option::None; // Panic: Out of physical memory!
        }
        self.current_address = next_address;
        core::option::Option::Some(aligned_address)
    }
}

/// Allocator for disk nodes.
///
/// # Fields
/// - `current_lba`: The current LBA to allocate from.
/// - `total_lbas`: The total number of LBAs available for allocation.
///
/// # Example
/// ```no_run
/// use ra::fs::nvme::allocator::DiskAllocator;
/// let mut disk_allocator = DiskAllocator::new(1000);
/// let lba = disk_allocator.allocate_node().expect("Allocation failed");
/// assert_eq!(lba, 8, "LBA is not as expected!");
/// ```
pub struct DiskAllocator {
    current_lba: u64,
    total_lbas: u64,
}

impl DiskAllocator {
    /// Creates a new `DiskAllocator` with the given total number of LBAs.
    ///
    /// # Arguments
    /// * `total_lbas` - The total number of LBAs available for allocation.
    /// # Returns
    /// A new instance of `DiskAllocator`.
    ///
    /// # Example
    /// ```no_run
    /// use ra::fs::nvme::allocator::DiskAllocator;
    /// let mut disk_allocator = DiskAllocator::new(1000);
    /// let lba = disk_allocator.allocate_node().expect("Allocation failed");
    /// assert_eq!(lba, 8, "LBA is not as expected!");
    /// ```
    #[must_use]
    pub const fn new(total_lbas: u64) -> Self {
        Self {
            current_lba: 8,
            total_lbas,
        }
    }
    /// Allocates a disk node by returning the starting LBA of the allocated node.
    /// # Returns
    /// An `Option` containing the starting LBA of the allocated node, or `None` if the allocation exceeds the total LBAs.
    ///
    /// # Example
    /// ```no_run
    /// use ra::fs::nvme::allocator::DiskAllocator;
    /// let mut disk_allocator = DiskAllocator::new(1000);
    /// let lba = disk_allocator.allocate_node().expect("Allocation failed");
    /// assert_eq!(lba, 8, "LBA is not as expected!");
    /// ```
    pub const fn allocate_node(&mut self) -> core::option::Option<u64> {
        let start_lba = self.current_lba;
        let next_lba = start_lba + 8;

        if next_lba > self.total_lbas {
            return core::option::Option::None;
        }

        self.current_lba = next_lba;
        core::option::Option::Some(start_lba)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disk_allocator_bounds() {
        let mut allocator = DiskAllocator::new(1000);
        assert_eq!(allocator.allocate_node().unwrap(), 8);
        assert_eq!(allocator.allocate_node().unwrap(), 16);
    }

    #[test]
    fn test_alignement_pages_dma_virtuelles() {
        let mut alloc = Allocator::new(0x1005, 0x10000);
        let page = alloc.allocate_page().expect("Allocation echouee");
        assert_eq!(page % 4096, 0, "CRITIQUE : Page non alignee !");
    }
}
