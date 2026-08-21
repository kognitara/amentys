use core::clone::Clone;
use core::fmt::Debug;
use core::marker::Copy;
/// The `NvmeRegisters` struct represents the memory-mapped registers of an `Nvme` controller.
///
/// It is used to interact with the `Nvme` hardware and perform various operations such as reading capabilities, configuring the controller, and managing queues.
///
/// The struct is defined with a `#[repr(C)]` attribute to ensure that its memory layout is compatible with C, which is important for low-level hardware interactions.
///
/// The fields of the struct correspond to specific registers in the `Nvme` specification, and they are defined with appropriate types to match the expected sizes of the registers.
///
/// The `NvmeRegisters` struct is a crucial component for implementing `Nvme` functionality in a Rust-based operating system or low-level application, allowing direct access to the `Nvme` controller's capabilities and configuration.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NvmeRegisters {
    pub cap: u64,       // 0x00 - Capacités
    pub vs: u32,        // 0x08 - Version
    pub intms: u32,     // 0x0C - Interruptions
    pub intmc: u32,     // 0x10 - Interruptions
    pub cc: u32,        // 0x14 - Controller Configuration
    pub reserved1: u32, // 0x18
    pub csts: u32,      // 0x1C - Controller Status
    pub nssr: u32,      // 0x20 - Subsystem Reset
    pub aqa: u32,       // 0x24 - Admin Queue Attributes (Taille des files)
    pub asq: u64,       // 0x28 - Admin Submission Queue Base Address
    pub acq: u64,       // 0x30 - Admin Completion Queue Base Address
}
