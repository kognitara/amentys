/// The `NvmeCommand` struct represents a command to be sent to the `Nvme` controller. It is aligned to 64 bytes to match the `Nvme` specification for command structures. The fields of the struct correspond to the various components of an `Nvme` command, including the opcode, flags, command ID, namespace ID, and pointers to data and metadata.
#[repr(C, align(64))]
#[derive(Clone, Copy, Default)]
pub struct NvmeCommand {
    pub opcode: u8,
    pub flags: u8,
    pub command_id: u16,
    pub namespace_id: u32,
    pub reserved: u64,
    pub metadata_ptr: u64,
    pub data_ptr: [u64; 2], // PRP1 et PRP2 (Pointeurs de données physiques)
    pub command_specific: [u32; 6],
}
/// The `NvmeCompletion` struct represents a completion entry returned by the `Nvme` controller after processing a command. It is aligned to 16 bytes to match the `Nvme` specification for completion structures. The fields of the struct include command-specific information, reserved space, submission queue head pointer, submission queue ID, command ID, and status information.
#[repr(C, align(16))]
#[derive(Clone, Copy, Default)]
pub struct NvmeCompletion {
    pub command_specific: u32,
    pub reserved: u32,
    pub sq_head: u16,
    pub sq_id: u16,
    pub command_id: u16,
    pub status: u16,
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::{align_of, size_of};

    #[test]
    fn test_geometrie_nvme_command() {
        // Le contrôleur `Nvme` s'attend à lire EXACTEMENT 64 octets.
        assert_eq!(
            size_of::<NvmeCommand>(),
            64,
            "CRITIQUE : NvmeCommand ne fait pas 64 octets !"
        );
        // L'alignement en mémoire doit correspondre à la ligne de cache (64 octets).
        assert_eq!(
            align_of::<NvmeCommand>(),
            64,
            "CRITIQUE : NvmeCommand n'est pas aligné sur 64 octets !"
        );
    }

    #[test]
    fn test_geometrie_nvme_completion() {
        // La file de complétion (CQ) utilise des entrées de 16 octets strictement.
        assert_eq!(
            size_of::<NvmeCompletion>(),
            16,
            "CRITIQUE : NvmeCompletion ne fait pas 16 octets !"
        );
        assert_eq!(
            align_of::<NvmeCompletion>(),
            16,
            "CRITIQUE : NvmeCompletion n'est pas aligné sur 16 octets !"
        );
    }
}
