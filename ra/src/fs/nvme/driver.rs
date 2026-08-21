use crate::fs::nvme::queue::{NvmeCommand, NvmeCompletion};
use crate::fs::nvme::registers::NvmeRegisters;
use crate::fs::nvme::{allocator::Allocator, bus::find_all_nvme};
use core::ptr::{read_volatile, write_volatile};
use heapless::Vec;
use jinshu::storage::BlockDevice;
use spin::Mutex;

/// L'état interne des files d'attente qui doit être muté à chaque commande
pub struct NvmeQueueState {
    sq_tail: u16,
    cq_head: u16,
    phase_tag: u16,
}
/// `Nvme` Driver structure that holds the necessary information for interacting with an `Nvme` device.
///
/// # Fields
/// * `registers`: Pointer to the `Nvme` registers.
/// * `admin_sub_virt_addr`: Virtual address of the Admin Submission Queue (ASQ
/// * `admin_cmp_virt_addr`: Virtual address of the Admin Completion Queue (ACQ).
/// * `sq0_tdbl`: Pointer to the Submission Queue 0 Tail Doorbell register.
/// * `cq0_hdbl`: Pointer to the Completion Queue 0 Head Doorbell register
/// * `total_lbas`: Total number of logical blocks (LBAs) on the `Nvme` device.
/// * `queue_state`: Mutex-protected state of the submission and completion queues, including the tail and head pointers and the phase tag.
pub struct NvmeDriver {
    // Pointeurs et adresses : immuables
    registers: *mut NvmeRegisters,
    admin_sub_virt_addr: u64,
    admin_cmp_virt_addr: u64,
    sq0_tdbl: *mut u32,
    cq0_hdbl: *mut u32,
    pub total_lbas: u64,
    queue_state: Mutex<NvmeQueueState>,
}

impl BlockDevice for NvmeDriver {
    fn read_node(&self, lba: u64, dest_phys_addr: u64) {
        let cmd = NvmeCommand {
            opcode: 0x02, // READ
            namespace_id: 1,
            data_ptr: [dest_phys_addr, 0], // L'adresse physique fournie par Jinshu
            command_specific: [
                (lba & 0xFFFF_FFFF) as u32,
                (lba >> 32) as u32,
                7, // Lecture de 8 blocs = 4096 octets
                0,
                0,
                0,
            ],
            ..NvmeCommand::default()
        };

        // Magie : on peut appeler une méthode qui mute le matériel tout en étant immuable !
        self.submit_admin_command(cmd);
    }

    fn write_node(&self, lba: u64, data_phys_addr: u64) {
        let cmd = NvmeCommand {
            opcode: 0x01, // WRITE
            namespace_id: 1,
            data_ptr: [data_phys_addr, 0],
            command_specific: [
                (lba & 0xFFFF_FFFF) as u32,
                (lba >> 32) as u32,
                7, // Écriture de 8 blocs = 4096 octets
                0,
                0,
                0,
            ],
            ..NvmeCommand::default()
        };

        self.submit_admin_command(cmd);
    }
}
impl NvmeDriver {
    /// Returns a mutable pointer to the `Nvme` registers.
    ///
    #[must_use]
    pub const fn reg(&self) -> *mut NvmeRegisters {
        self.registers
    }
    /// Soumet une commande au SSD en utilisant le Mutex pour muter les index
    fn submit_admin_command(&self, mut cmd: NvmeCommand) -> NvmeCompletion {
        let mut state = self.queue_state.lock();
        cmd.command_id = state.sq_tail;

        // SAFETY: L'accès à la mémoire DMA et aux registres MMIO est strictement contrôlé et séquentiel.
        unsafe {
            // 1. Écriture dans la Submission Queue (RAM)
            let sq_ptr =
                (self.admin_sub_virt_addr as *mut NvmeCommand).add(usize::from(state.sq_tail));
            core::ptr::write(sq_ptr, cmd);
            core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

            // 2. Incrémentation
            state.sq_tail += 1;
            if state.sq_tail == 64 {
                state.sq_tail = 0;
            }

            // 3. DOORBELL (Sonne le SSD)
            core::ptr::write_volatile(self.sq0_tdbl, u32::from(state.sq_tail));

            // 4. POLLING de la Completion Queue
            let cq_ptr =
                (self.admin_cmp_virt_addr as *mut NvmeCompletion).add(usize::from(state.cq_head));
            let status_ptr = core::ptr::addr_of!((*cq_ptr).status);

            loop {
                // On lit uniquement le champ status en volatile pour éviter le cache du CPU
                let status = core::ptr::read_volatile(status_ptr);
                let p = (status & 0x0001) as u16;

                if p == state.phase_tag {
                    // Lecture de la complétion entière de manière sécurisée
                    let completion = core::ptr::read(cq_ptr);

                    state.cq_head += 1;
                    if state.cq_head == 64 {
                        state.cq_head = 0;
                        state.phase_tag ^= 1;
                    }

                    core::ptr::write_volatile(self.cq0_hdbl, u32::from(state.cq_head));
                    return completion;
                }

                core::hint::spin_loop();
            }
        }
    }
    /// Returns the virtual address of the Admin Submission Queue (ASQ).
    #[must_use]
    pub const fn asq(&self) -> u64 {
        self.admin_sub_virt_addr
    }
    /// Returns the virtual address of the Admin Completion Queue (ACQ).
    #[must_use]
    pub const fn acq(&self) -> u64 {
        self.admin_cmp_virt_addr
    }

    /// Returns a mutable pointer to the Submission Queue 0 Tail Doorbell register.
    #[must_use]
    pub const fn sq0_tdbl(&self) -> *mut u32 {
        self.sq0_tdbl
    }
    /// Returns a mutable pointer to the Completion Queue 0 Head Doorbell register.
    #[must_use]
    pub const fn cq0_hdbl(&self) -> *mut u32 {
        self.cq0_hdbl
    }

    /// Returns the total number of logical blocks (LBAs) on the `Nvme` device.
    #[must_use]
    pub const fn total_lbas(&self) -> u64 {
        self.total_lbas
    }

    /// Returns the model of the `Nvme` device as a byte array.
    /// # Arguments
    /// * `buffer_phys_addr` - The physical address of the buffer containing the Identify Controller data.
    /// * `phys_offset` - The physical offset to convert the physical address to a virtual address.
    /// # Returns
    /// A byte array containing the model of the `Nvme` device.
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset, admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let model = nvme_driver.get_disk_model(identify_buffer_phys, phys_offset);
    /// ```
    #[must_use]
    pub fn get_disk_model(&self, buffer_phys_addr: u64, phys_offset: u64) -> [u8; 40] {
        let buffer_virt = buffer_phys_addr + phys_offset;
        let model_ptr = (buffer_virt + 24) as *const u8;

        let mut model = [0u8; 40];
        for (i, byte) in model.iter_mut().enumerate() {
            *byte = unsafe {
                // Safety: Read the model string from the Identify Controller data structure
                core::ptr::read_volatile(model_ptr.add(i))
            };
        }
        model
    }
    /// Returns the serial number of the `Nvme` device as a byte array.
    /// # Arguments
    /// * `buffer_phys_addr` - The physical address of the buffer containing the Identify Controller data.
    /// * `phys_offset` - The physical offset to convert the physical address to a virtual address.
    /// # Returns
    /// A byte array containing the serial number of the `Nvme` device.
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with the actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with the actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset , admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let serial = nvme_driver.get_disk_serial(identify_buffer_phys, phys_offset);
    /// ```
    #[must_use]
    pub fn get_disk_serial(&self, buffer_phys_addr: u64, phys_offset: u64) -> [u8; 20] {
        let buffer_virt = buffer_phys_addr + phys_offset;
        let serial_ptr = (buffer_virt + 4) as *const u8;
        let mut serial = [0u8; 20];
        for (i, byte) in serial.iter_mut().enumerate() {
            *byte = unsafe {
                // Safety: Read the serial number from the Identify Controller data structure
                core::ptr::read_volatile(serial_ptr.add(i))
            };
        }
        serial
    }
    /// Returns the firmware revision of the `Nvme` device as a byte array.
    /// # Arguments
    /// * `buffer_phys_addr` - The physical address of the buffer containing the Identify Controller data.
    /// * `phys_offset` - The physical offset to convert the physical address to a virtual address.
    /// # Returns
    /// A byte array containing the firmware revision of the `Nvme` device.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with the actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with the actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset , admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let firmware = nvme_driver.get_disk_firmware(identify_buffer_phys, phys_offset);
    /// ```
    #[must_use]
    pub fn get_disk_firmware(&self, buffer_phys_addr: u64, phys_offset: u64) -> [u8; 8] {
        let buffer_virt = buffer_phys_addr + phys_offset;
        let firmware_ptr = (buffer_virt + 64) as *const u8;
        let mut firmware = [0u8; 8];
        for (i, byte) in firmware.iter_mut().enumerate() {
            *byte = unsafe {
                // Safety: Read the firmware revision from the Identify Controller data structure
                core::ptr::read_volatile(firmware_ptr.add(i))
            };
        }
        firmware
    }

    /// Returns the size of the disk in logical blocks (LBAs).
    /// # Arguments
    /// * `buffer_phys_addr` - The physical address of the buffer containing the Identify Namespace data.
    /// * `phys_offset` - The physical offset to convert the physical address to a virtual address.
    /// # Returns
    /// The size of the disk in logical blocks (LBAs).
    ///
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    ///
    /// The caller must ensure that the provided buffer physical address is valid and points to a memory region
    ///
    /// that can be safely read from.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with the actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with the actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset , admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let disk_size_lba = nvme_driver.get_disk_size_lba(identify_buffer_phys, phys_offset);
    /// ```
    #[must_use]
    pub fn get_disk_size_lba(&self, buffer_phys_addr: u64, phys_offset: u64) -> u64 {
        let buffer_virt = buffer_phys_addr + phys_offset;
        unsafe {
            // Safety: Read the total number of logical blocks (LBAs) from the Identify Namespace data structure
            core::ptr::read_volatile(buffer_virt as *const u64)
        }
    }
    /// Initializes an `Nvme` device with the given BAR0 address, physical offset, and queue addresses.
    /// # Arguments
    /// * `bar0_address` - A reference to the BAR0 address of the `Nvme` device.
    /// * `phys_offset` - The physical offset to convert physical addresses to virtual addresses.
    /// * `admin_sub_phys_addr` - The physical address of the Admin Submission Queue
    /// * `admin_cmp_phys_addr` - The physical address of the Admin Completion Queue
    /// # Returns
    /// An `Option` containing an `NvmeDriver` instance if initialization is successful, or `None` if it fails.
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    /// The caller must ensure that the provided addresses are valid and that the `NVMe` device is properly configured.
    ///
    /// # Panics
    /// Panics if the provided BAR0 address is invalid or if the `Nvme` device cannot be initialized.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with the actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with the actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset, admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// ```
    #[must_use]
    pub fn initialize(
        bar0_address: &usize,
        phys_offset: u64,
        admin_sub_phys_addr: u64,
        admin_cmp_phys_addr: u64,
    ) -> core::option::Option<Self> {
        let virtual_address = (*bar0_address as u64) + phys_offset;
        let registers = virtual_address as *mut NvmeRegisters;
        let sq0_tdbl = (virtual_address + 0x1000) as *mut u32;
        let cq0_hdbl = (virtual_address + 0x1004) as *mut u32;
        let admin_sub_virt_addr = admin_sub_phys_addr + phys_offset;
        let admin_cmp_virt_addr = admin_cmp_phys_addr + phys_offset;

        unsafe {
            // Safety: Deactivation of the `Nvme` controller
            let mut cc = read_volatile(&raw const (*registers).cc);
            cc &= !1;
            write_volatile(&raw mut (*registers).cc, cc);
            loop {
                if (read_volatile(&raw const (*registers).csts) & 1) == 0 {
                    break;
                }
            }

            let admin_sub_clear = admin_sub_virt_addr as *mut u64;
            let admin_cmp_clear = admin_cmp_virt_addr as *mut u64;
            for i in 0..512 {
                write_volatile(admin_sub_clear.add(i), 0);
                write_volatile(admin_cmp_clear.add(i), 0);
            }

            write_volatile(&raw mut (*registers).aqa, (63 << 16) | 63);
            write_volatile(&raw mut (*registers).asq, admin_sub_phys_addr);
            write_volatile(&raw mut (*registers).acq, admin_cmp_phys_addr);

            cc |= 1;
            cc |= (6 << 16) | (4 << 20);
            write_volatile(&raw mut (*registers).cc, cc);
            loop {
                if (read_volatile(&raw const (*registers).csts) & 1) == 1 {
                    break;
                }
            }
        }
        core::option::Option::Some(Self {
            registers,
            admin_sub_virt_addr,
            admin_cmp_virt_addr,
            sq0_tdbl,
            cq0_hdbl,
            total_lbas: 0,
            queue_state: Mutex::new(NvmeQueueState {
                sq_tail: 0,
                cq_head: 0,
                phase_tag: 1,
            }),
        })
    }

    /// Clears the command structure at the given pointer by writing zeros to the first 16 u32 entries.
    /// # Arguments
    /// * `ptr` - A mutable pointer to the command structure to be cleared.
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    /// The caller must ensure that the provided pointer is valid and points to a memory
    /// region that can be safely written to.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset, admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let command_ptr = nvme_driver.asq() as *mut u32; // Replace with the actual command structure pointer
    /// NvmeDriver::clear_command(command_ptr);
    /// ```
    fn clear_command(ptr: *mut u32) {
        unsafe {
            // Safety: Clear the command structure by writing zeros to the first 16 u32 entries
            for i in 0..16 {
                write_volatile(ptr.add(i), 0);
            }
        }
    }
    /// Sends an Identify Controller command to the `Nvme` device, using the provided buffer physical address to store the response.
    /// # Arguments
    /// * `buffer_phys_addr` - The physical address of the buffer where the Identify Controller response will be stored.
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    /// The caller must ensure that the provided buffer physical address is valid and points to a memory region that can be safely written to.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with the actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with the actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset , admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let identify_buffer_phys = 0x0000_0000; // Replace with the actual physical address of the buffer for the Identify Controller response
    /// nvme_driver.identify_controller(identify_buffer_phys);
    /// ```
    pub fn identify_controller(&self, buffer_phys_addr: u64) {
        let ptr = self.admin_sub_virt_addr as *mut u32;
        unsafe {
            // Safety: Clear the command structure before setting up the Identify Controller command
            Self::clear_command(ptr);
            write_volatile(ptr.add(0), (1 << 16) | 0x06);
            write_volatile(ptr.add(6), (buffer_phys_addr & 0xFFFF_FFFF) as u32);
            write_volatile(ptr.add(7), (buffer_phys_addr >> 32) as u32);
            write_volatile(ptr.add(10), 1);
        }
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        unsafe {
            // Safety: Write to the Submission Queue 0 Tail Doorbell register to notify the `Nvme` controller of the new command
            write_volatile(self.sq0_tdbl, 1);
        }
    }
    /// Sends an Identify Namespace command to the `Nvme` device, using the provided buffer physical address to store the response.
    /// # Arguments
    /// * `buffer_phys_addr` - The physical address of the buffer where the Identify Namespace  response will be stored.
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    /// The caller must ensure that the provided buffer physical address is valid and points to a memory region that can be safely written to.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with the actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with the actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset , admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let identify_buffer_phys = 0x0000_0000; // Replace with the actual physical address of the buffer for the Identify Namespace response
    /// nvme_driver.identify_namespace(identify_buffer_phys);
    /// ```
    ///
    pub fn identify_namespace(&self, buffer_phys_addr: u64) {
        let ptr = (self.admin_sub_virt_addr as *mut u32).wrapping_add(16);
        unsafe {
            // Safety: Clear the command structure before setting up the Identify Namespace command
            Self::clear_command(ptr);
            write_volatile(ptr.add(0), (2 << 16) | 0x06);
            write_volatile(ptr.add(1), 1);
            write_volatile(ptr.add(6), (buffer_phys_addr & 0xFFFF_FFFF) as u32);
            write_volatile(ptr.add(7), (buffer_phys_addr >> 32) as u32);
            write_volatile(ptr.add(10), 0);
        }
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        unsafe {
            // Safety: Write to the Submission Queue 0 Tail Doorbell register to notify the `Nvme` controller of the new command
            write_volatile(self.sq0_tdbl, 2);
        }
    }
    /// Creates an I/O Completion Queue (CQ) for the `NVMe` device at the specified physical address.
    /// # Arguments
    /// * `cq1_phys_addr` - The physical address where the I/O Completion Queue will be created.
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    ///
    /// The caller must ensure that the provided physical address is valid and points to a memory region that can be safely written to.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset, admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let cq1_phys_addr = 0x0000_0000; // Replace with actual physical address for the I/O Completion Queue
    /// nvme_driver.create_io_cq(cq1_phys_addr);
    /// ```
    pub fn create_io_cq(&self, cq1_phys_addr: u64) {
        let ptr = (self.admin_sub_virt_addr as *mut u32).wrapping_add(32);
        unsafe {
            // Safety: Clear the command structure before setting up the I/O Completion Queue command
            Self::clear_command(ptr);
            write_volatile(ptr.add(0), (3 << 16) | 0x05);
            write_volatile(ptr.add(6), (cq1_phys_addr & 0xFFFF_FFFF) as u32);
            write_volatile(ptr.add(7), (cq1_phys_addr >> 32) as u32);
            write_volatile(ptr.add(10), (63 << 16) | 1);
            write_volatile(ptr.add(11), 1);
        }
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        unsafe {
            // Safety: Write to the Submission Queue 0 Tail Doorbell register to notify the `Nvme` controller of the new command
            write_volatile(self.sq0_tdbl, 3);
        }
    }
    /// Creates an I/O Submission Queue (SQ) for the `NVMe` device at the specified physical address.
    /// # Arguments
    /// * `sq1_phys_addr` - The physical address where the I/O Submission Queue will be created.
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    ///
    /// The caller must ensure that the provided physical address is valid and points to a memory region that can be safely written to.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset, admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let sq1_phys_addr = 0x0000_0000; // Replace with actual physical address for the I/O Submission Queue
    /// nvme_driver.create_io_sq(sq1_phys_addr);
    /// ```
    pub fn create_io_sq(&self, sq1_phys_addr: u64) {
        let ptr = (self.admin_sub_virt_addr as *mut u32).wrapping_add(48);
        unsafe {
            // Safety: Clear the command structure before setting up the I/O Submission Queue command
            Self::clear_command(ptr);
            write_volatile(ptr.add(0), (4 << 16) | 0x01);
            write_volatile(ptr.add(6), (sq1_phys_addr & 0xFFFF_FFFF) as u32);
            write_volatile(ptr.add(7), (sq1_phys_addr >> 32) as u32);
            write_volatile(ptr.add(10), (63 << 16) | 1);
            write_volatile(ptr.add(11), (1 << 16) | 1);
        }
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        unsafe {
            // Safety: Write to the Submission Queue 0 Tail Doorbell register to notify the `Nvme` controller of the new command
            write_volatile(self.sq0_tdbl, 4);
        }
    }
    /// Sends a Read command to the `NVMe` device to read data from the specified logical block address (LBA) into the provided data buffer.
    /// # Arguments
    /// * `sq1_virt_addr` - The virtual address of the I/O Submission Queue (SQ1) where the Read command will be placed.
    /// * `data_phys_addr` - The physical address of the data buffer where the read data will be stored.
    /// * `lba` - The logical block address (LBA) from which to read data.
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    ///
    /// The caller must ensure that the provided virtual and physical addresses are valid and that the `NVMe` device is properly configured to handle the Read command.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with the actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with the actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset , admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let sq1_virt_addr = 0x0000_0000; // Replace with the actual virtual address of the I/O Submission Queue
    /// let data_phys_addr = 0x0000_0000; // Replace with the actual physical address of the data buffer
    /// let lba = 0; // Replace with the desired logical block address to read
    /// nvme_driver.read_lba(sq1_virt_addr, data_phys_addr, lba);
    /// ```
    pub fn read_lba(&self, sq1_virt_addr: u64, data_phys_addr: u64, lba: u64) {
        let ptr = (sq1_virt_addr as *mut u32).wrapping_add(16);
        unsafe {
            // Safety: Clear the command structure before setting up the Read command
            Self::clear_command(ptr);
            write_volatile(ptr.add(0), (2 << 16) | 0x02);
            write_volatile(ptr.add(1), 1);
            write_volatile(ptr.add(6), (data_phys_addr & 0xFFFF_FFFF) as u32);
            write_volatile(ptr.add(7), (data_phys_addr >> 32) as u32);
            write_volatile(ptr.add(10), (lba & 0xFFFF_FFFF) as u32);
            write_volatile(ptr.add(11), (lba >> 32) as u32);
            write_volatile(ptr.add(12), 7);
        }
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        unsafe {
            // Safety: Write to the Submission Queue 1 Tail Doorbell register to notify the `NVMe` controller of the new command
            write_volatile((self.registers as u64 + 0x1008) as *mut u32, 1);
        }
    }
    /// Sends a Write command to the `NVMe` device to write data from the provided data buffer to the specified logical block address (LBA).
    /// # Arguments
    /// * `sq1_virt_addr` - The virtual address of the I/O Submission Queue (SQ1) where the Write command will be placed.
    /// * `data_phys_addr` - The physical address of the data buffer containing the data to be written.
    /// * `lba` - The logical block address (LBA) to which the data will be written.
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    ///
    /// The caller must ensure that the provided virtual and physical addresses are valid and that the `NVMe` device is properly configured to handle the Write command.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with the actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with the actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset , admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let sq1_virt_addr = 0x0000_0000; // Replace with the actual virtual address of the I/O Submission Queue
    /// let data_phys_addr = 0x0000_0000; // Replace with the actual physical address of the data buffer
    /// let lba = 0; // Replace with the desired logical block address to write
    /// nvme_driver.write_lba(sq1_virt_addr, data_phys_addr, lba);
    /// ```
    pub fn write_lba(&self, sq1_virt_addr: u64, data_phys_addr: u64, lba: u64) {
        let ptr = sq1_virt_addr as *mut u32;
        unsafe {
            // Safety: Clear the command structure before setting up the Write command
            Self::clear_command(ptr);
            write_volatile(ptr.add(0), (1 << 16) | 0x01);
            write_volatile(ptr.add(1), 1);
            write_volatile(ptr.add(6), (data_phys_addr & 0xFFFF_FFFF) as u32);
            write_volatile(ptr.add(7), (data_phys_addr >> 32) as u32);
            write_volatile(ptr.add(10), (lba & 0xFFFF_FFFF) as u32);
            write_volatile(ptr.add(11), (lba >> 32) as u32);
            write_volatile(ptr.add(12), 7);
        }
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        unsafe {
            // Safety: Write to the Submission Queue 1 Tail Doorbell register to notify the `NVMe` controller of the new command
            write_volatile((self.registers as u64 + 0x1008) as *mut u32, 1);
        }
    }
    /// Waits for the completion of a command in the Admin Completion Queue (ACQ) at the specified index.
    /// # Arguments
    /// * `index` - The index of the command in the Admin Completion Queue to wait for.
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    /// The caller must ensure that the provided index is valid and that the Admin Completion Queue is properly configured to handle command completions.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds for the Admin Completion Queue.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with the actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with the actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset , admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// nvme_driver.wait_for_completion(0); // Wait for the completion of the command at index 0 in the Admin Completion Queue
    /// ```
    pub fn wait_for_completion(&self, index: usize) {
        let acq_ptr = self.admin_cmp_virt_addr as *mut u32;
        unsafe {
            // Safety: Wait for the command completion by polling the status dword in the Admin Completion Queue
            loop {
                let status_dword = read_volatile(acq_ptr.add(index * 4 + 3));
                if ((status_dword >> 16) & 0x01) == 1 {
                    write_volatile(self.cq0_hdbl, u32::try_from(index + 1).unwrap_or(0));
                    break;
                }
            }
        }
    }
    /// Waits for the completion of a command in the I/O Completion Queue (CQ1) at the specified index.
    ///
    /// # Arguments
    /// * `cq1_virt_addr` - The virtual address of the I/O Completion
    /// * `index` - The index of the command in the I/O Completion Queue to wait for.
    /// # Safety
    /// This function performs raw pointer dereferencing and volatile memory access, which can lead to undefined behavior if used incorrectly.
    /// The caller must ensure that the provided virtual address and index are valid and that the I/O Completion Queue is properly configured to handle command completions.
    /// # Panics
    ///
    /// Panics if the index is out of bounds for the I/O Completion Queue.
    ///
    /// # Example
    /// ```no_run
    /// use crate::fs::nvme::driver::NvmeDriver;
    /// let bar0_address = 0x0000_0000; // Replace with the actual BAR0 address of the NVMe device
    /// let phys_offset = 0x0000_0000; // Replace with the actual physical offset for your system
    /// let admin_sub_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Submission Queue
    /// let admin_cmp_phys_addr = 0x0000_0000; // Replace with the actual physical address of the Admin Completion Queue
    /// let nvme_driver = NvmeDriver::initialize(&bar0_address, phys_offset , admin_sub_phys_addr, admin_cmp_phys_addr).unwrap();
    /// let cq1_virt_addr = 0x0000_0000; // Replace with the actual virtual address of the I/O Completion Queue
    /// nvme_driver.wait_for_io_completion(cq1_virt_addr, 0); // Wait for the completion of the command at index 0 in the I/O Completion Queue
    /// ```
    pub fn wait_for_io_completion(&self, cq1_virt_addr: u64, index: usize) {
        let cq_ptr = cq1_virt_addr as *mut u32;
        unsafe {
            // Safety: Wait for the command completion by polling the status dword in the I/O Completion Queue
            loop {
                let status_dword = read_volatile(cq_ptr.add(index * 4 + 3));
                if ((status_dword >> 16) & 0x01) == 1 {
                    write_volatile(
                        (self.registers as u64 + 0x100C) as *mut u32,
                        u32::try_from(index + 1).unwrap_or(0),
                    );
                    break;
                }
            }
        }
    }
}

/// Initialise tous les contrôleurs `NVMe` détectés sur le bus PCI.
///
/// # Panics
///
/// Cette fonction ne panique pas. Si l'allocation mémoire des files d'administration
/// échoue, le périphérique `NVMe` concerné est silencieusement ignoré.
#[must_use]
pub fn init_all_nvme_devices(phys_offset: u64, allocator: &mut Allocator) -> Vec<NvmeDriver, 32> {
    let nvme_bars = find_all_nvme();
    let mut drivers = Vec::new();

    for bar in nvme_bars {
        let registers = (bar as u64 + phys_offset) as *mut NvmeRegisters;

        // SAFETY: Manipulation des registres de contrôle MMIO du périphérique PCI NVMe.
        unsafe {
            let mut cc = core::ptr::read_volatile(&raw const (*registers).cc);
            cc &= !1;
            core::ptr::write_volatile(&raw mut (*registers).cc, cc);

            while (core::ptr::read_volatile(&raw const (*registers).csts) & 1) != 0 {
                core::hint::spin_loop();
            }

            // CORRECTION : Plus de expect(), et renommage pour éviter la similarité
            let Some(admin_sub_phys) = allocator.allocate_page() else {
                continue;
            };
            let Some(admin_cmp_phys) = allocator.allocate_page() else {
                continue;
            };

            core::ptr::write_bytes((admin_sub_phys + phys_offset) as *mut u8, 0, 4096);
            core::ptr::write_bytes((admin_cmp_phys + phys_offset) as *mut u8, 0, 4096);

            core::ptr::write_volatile(&raw mut (*registers).aqa, 0x003F_003F);
            core::ptr::write_volatile(&raw mut (*registers).asq, admin_sub_phys);
            core::ptr::write_volatile(&raw mut (*registers).acq, admin_cmp_phys);

            cc |= (6 << 16) | (4 << 20) | 1;
            core::ptr::write_volatile(&raw mut (*registers).cc, cc);

            while (core::ptr::read_volatile(&raw const (*registers).csts) & 1) == 0 {
                core::hint::spin_loop();
            }

            let cap = core::ptr::read_volatile(&raw const (*registers).cap);
            let dstrd = ((cap >> 32) & 0b1111) as u64;
            let stride = 1 << (2 + dstrd);

            let sq0_tdbl = (bar as u64 + phys_offset + 0x1000) as *mut u32;
            let cq0_hdbl = (bar as u64 + phys_offset + 0x1000 + stride) as *mut u32;
            let driver = NvmeDriver {
                registers,
                admin_sub_virt_addr: admin_sub_phys + phys_offset,
                admin_cmp_virt_addr: admin_cmp_phys + phys_offset,
                sq0_tdbl,
                cq0_hdbl,
                total_lbas: 100_000_000,
                queue_state: Mutex::new(NvmeQueueState {
                    sq_tail: 0,
                    cq_head: 0,
                    phase_tag: 1,
                }),
            };

            drivers.push(driver).ok();
        }
    }
    drivers
}
