use heapless::Vec;
use x86_64::instructions::port::Port;

/// Read a 32-bit PCI register
///
/// # Safety
/// This function performs raw I/O port access, which can lead to undefined behavior if used incorrectly.
///
/// The caller must ensure that the bus, slot, function, and offset parameters are valid and that the PCI device is present.
///
/// # Arguments
/// * `bus` - The PCI bus number (0-255).
/// * `slot` - The PCI slot number (0-31).
/// * `function` - The PCI function number (0-7).
/// * `offset` - The offset within the PCI configuration space (0-255).
/// # Returns
/// The 32-bit value read from the specified PCI register.
///
/// # Example
///
/// ```no_run
/// use ra::fs::nvme::bus::read_pci_reg;
/// let vendor_id = unsafe { read_pci_reg(0, 0, 0, 0x00) };
/// println!("Vendor ID: {:04x}", vendor_id & 0xFFFF);
/// ```
///
unsafe fn read_pci_reg(bus: u8, slot: u8, function: u8, offset: u8) -> u32 {
    let address = 0x8000_0000
        | ((u32::from(bus)) << 16)
        | ((u32::from(slot)) << 11)
        | ((u32::from(function)) << 8)
        | ((u32::from(offset)) & 0xFC);

    // Use the standard PCI ports of the x86 architecture
    let mut port_address = Port::<u32>::new(0xCF8);
    let mut port_data = Port::<u32>::new(0xCFC);

    unsafe {
        // Safety: Write the address to the PCI configuration address port
        port_address.write(address);
    };
    unsafe {
        // Safety: Read the data from the PCI configuration data port
        port_data.read()
    }
}

/// Finds the first `Nvme` device on the PCI bus and returns its BAR0 address.
/// # Returns
/// An `Option` containing the BAR0 address of the first `Nvme` device found, or `None` if no `Nvme` device is found.
/// # Safety
/// This function performs raw I/O port access, which can lead to undefined behavior if used incorrectly.
///
/// The caller must ensure that the PCI bus is properly initialized and that the system supports `Nvme` devices before calling this function.
///
/// # Example
///
/// ```no_run
/// use ra::fs::nvme::bus::find_nvme;
/// if let Some(bar0) = find_nvme() {
///     println!("Found NVMe device at BAR0 address: {:x}", bar0);
/// } else {
///     println!("No NVMe device found.");
/// }
/// ```
#[must_use]
pub fn find_nvme() -> core::option::Option<usize> {
    unsafe {
        // Safety: for each slot (0-31) on the PCI bus, check for `Nvme` devices
        for slot in 0..32 {
            let vendor = read_pci_reg(0, slot, 0, 0x00);
            if vendor == 0xFFFF_FFFF {
                continue; // No device present in this slot
            }

            let class_info = read_pci_reg(0, slot, 0, 0x08);
            let class = (class_info >> 24) & 0xFF;
            let sub_class = (class_info >> 16) & 0xFF;

            // Class 0x01 (Mass Storage), Sub-class 0x08 (Non-Volatile Memory)
            if class == 0x01 && sub_class == 0x08 {
                let bar0 = read_pci_reg(0, slot, 0, 0x10);
                // On masque les bits de statut pour avoir l'adresse physique pure
                return core::option::Option::Some((bar0 & 0xFFFF_FFF0) as usize);
            }
        }
    }
    core::option::Option::None
}
/// Finds all `Nvme` devices on the PCI bus and returns their BAR0 addresses.
/// # Safety
/// This function performs raw I/O port access, which can lead to undefined behavior if used incorrectly.
/// The caller must ensure that the PCI bus is properly initialized and that the system supports `Nvme` devices before calling this function.
///
/// # Returns
/// A `Vec` containing the BAR0 addresses of all `Nvme` devices found on the PCI bus.
///
/// The vector has a maximum capacity of 32, as there can be at most 32 devices on a single PCI bus.
///
/// # Example
///
/// ```no_run
/// use ra::fs::nvme::bus::find_all_nvme;
/// let nvme_devices = find_all_nvme();
/// for bar0 in nvme_devices {
///     println!("Found NVMe device at BAR0 address: {bar0:x}");
/// }
/// ```
#[must_use]
pub fn find_all_nvme() -> Vec<usize, 32> {
    let mut nvme_list: Vec<usize, 32> = Vec::new();
    let bus = 0; // On QEMU the main bus is bus 0

    unsafe {
        // Safety: for each slot (0-31) on the PCI bus, check for `Nvme` devices
        for slot in 0..32 {
            let vendor = read_pci_reg(bus, slot, 0, 0x00);
            if vendor == 0xFFFF_FFFF {
                continue;
            }

            for func in 0..8 {
                let vendor = read_pci_reg(bus, slot, func, 0x00);
                if vendor == 0xFFFF_FFFF {
                    continue;
                }

                let class_info = read_pci_reg(bus, slot, func, 0x08);
                let class = (class_info >> 24) & 0xFF;
                let sub_class = (class_info >> 16) & 0xFF;

                if class == 0x01 && sub_class == 0x08 {
                    // ==============================================================
                    // LE FIX MAGIQUE : AUTORISATION DU DMA (BUS MASTERING)
                    // ==============================================================
                    // On lit le registre de Commande PCI (Offset 0x04)
                    let mut cmd = read_pci_reg(bus, slot, func, 0x04);
                    // On active le bit 1 (Memory Space) et le bit 2 (Bus Master)
                    cmd |= 0x0000_0006;

                    // On forge l'adresse pour réécrire dans le registre 0x04
                    let cmd_address = 0x8000_0000
                        | ((u32::from(bus)) << 16)
                        | ((u32::from(slot)) << 11)
                        | ((u32::from(func)) << 8)
                        | 0x04;

                    let mut port_address = Port::<u32>::new(0xCF8);
                    let mut port_data = Port::<u32>::new(0xCFC);
                    port_address.write(cmd_address);
                    port_data.write(cmd); // Autorisation accordée !
                    // ==============================================================

                    let bar0 = read_pci_reg(bus, slot, func, 0x10);
                    let mut bar_address = (bar0 & 0xFFFF_FFF0) as usize;

                    if (bar0 & 0b110) == 0b100 {
                        let bar1 = read_pci_reg(bus, slot, func, 0x14);
                        bar_address |= (bar1 as usize) << 32;
                    }

                    if bar_address != 0 {
                        nvme_list.push(bar_address).ok();
                    }
                }
            }
        }
    }
    nvme_list
}
