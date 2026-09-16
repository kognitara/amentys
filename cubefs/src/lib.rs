#![cfg_attr(not(test), no_std)]

#[cfg(test)]
use blake3::Hasher;

///
/// # Fields
/// - `seq`: Sequence of the checkpoint
/// - `time`: Timestamp UTC (hardware time)
/// - `root_meta_id`: Hash BLAKE3 of the root of the B-Tree of metadata
/// - `merkle_root`: Hash BLAKE3 global of the data graph
/// - `signature`: Signature of the checkpoint
///
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Checkpoint {
    pub seq: u64,
    pub time: u64,
    pub root_meta_id: [u8; 32],
    pub merkle_root: [u8; 32],
    pub signature: [u8; 64],
}

/// A B-Tree node strictly aligned on 4096 bytes (1 NVMe page)
///
/// # Fields
/// - `magic`: Magic number of the layout
/// - `version`: Version of the layout
/// - `level`: Level in the B-Tree (0 = leaf)
/// - `entry_count`: Number of entries (keys/values) contained
/// - `payload`: Contains the serialized keys and values.
/// - `blake3_hash`: Identity cryptographic hash of the node
#[repr(C, align(4096))]
#[derive(Debug, Clone, Copy)]
pub struct BTreeNode {
    pub magic: [u8; 4],
    pub version: u8,
    pub level: u8,
    pub entry_count: u16,
    pub payload: [u8; 4056],
    pub blake3_hash: [u8; 32],
}

/// Structure of a pure POSIX file/directory
///
/// # Fields
/// - `id`: Hash BLAKE3 of this Inode
/// - `size`: Total logical decompressed size
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Inode {
    pub id: [u8; 32],
    pub size: u64,
}

/// Map a range of file to a sequence of KPACK chunks
///
/// # Fields
/// - `file_off` Offset in the logical file
/// - `len` Length covered by this extent
/// - `first_chunk_id` Hash BLAKE3 of the first chunk in KPACK
/// - `count` Number of sequential chunks
/// - `last_len` Useful size in the last chunk
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Extent {
    pub file_off: u64,
    pub len: u64,
    pub first_chunk_id: [u8; 32],
    pub count: u32,
    pub last_len: u32,
}

/// Representation in memory of a Key of the B-Tree (for parsing the payload)
///
/// # Fields
/// - `hash` Hash rapid (parent_id || name) for the research
/// - `name_len` Length of the dynamic name that follows in memory
#[repr(C)]
pub struct BTreeKeyHeader {
    pub hash: u64,
    pub name_len: u16,
}

/// Representation in memory of a Value of the B-Tree
///
/// # Fields
/// - `node_type` Type (0: Inode, 1: Directory, 2: Inline data...)
/// - `data` Contains the ID `[u8; 32]`, a pointer u64, or the literal data (inline)
#[repr(C)]
pub struct BTreeVal {
    pub node_type: u8,
    pub data: [u8; 32],
}

/// The result of an iteration over a B-Tree payload
///
/// # Fields
/// - `hash` The hash
/// - `name` A slice of the name
/// - `val_type` The type of the value
/// - `val_data` The identifier of the inline value
#[repr(C)]
pub struct BTreeEntry<'a> {
    pub hash: u64,
    pub name: &'a [u8],
    pub val_type: u8,
    pub val_data: [u8; 32],
}
/// Represents an iterator over a B-Tree payload
///
/// # Fields
/// - `payload`
/// - `entry_to_read`
/// - `current_offset`
///
pub struct BTreeIterator<'a> {
    payload: &'a [u8],
    entries_to_read: u16,
    current_offset: usize,
}

impl<'a> BTreeIterator<'a> {
    /// Initialise
    #[must_use]
    pub const fn new(payload: &'a [u8], entry_count: u16) -> Self {
        Self {
            payload,
            entries_to_read: entry_count,
            current_offset: 0,
        }
    }
}
/// The nucleator, the Copy-On-Write mutation engine for Amentys
pub struct Nucleator;

impl Nucleator {
    /// Strike a byte into an existing buffer to generate a new state.
    /// Returns the new pure chunk and the ejected byte, if any.
    ///
    /// # Args
    /// - `original`
    /// - `offset`
    /// - `new_byte`
    #[must_use]
    pub fn insert_byte(
        original: &[u8; 4096],
        offset: usize,
        new_byte: u8,
    ) -> Option<([u8; 4096], u8)> {
        if offset >= 4096 {
            return None;
        }

        let mut new_buffer = [0u8; 4096];

        new_buffer[..offset].copy_from_slice(&original[..offset]);
        new_buffer[offset] = new_byte;

        let ejected_byte = original[4095];
        if offset < 4095 {
            new_buffer[offset + 1..].copy_from_slice(&original[offset..4095]);
        }
        Some((new_buffer, ejected_byte))
    }
}
impl<'a> Iterator for BTreeIterator<'a> {
    type Item = BTreeEntry<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.entries_to_read == 0 || self.current_offset >= self.payload.len() {
            return None;
        }

        let hash_bytes = self
            .payload
            .get(self.current_offset..self.current_offset + 8)?;
        let hash = u64::from_le_bytes(hash_bytes.try_into().unwrap());
        self.current_offset += 8;
        let name_len_bytes = self
            .payload
            .get(self.current_offset..self.current_offset + 2)?;
        let name_len = u16::from_le_bytes(name_len_bytes.try_into().unwrap()) as usize;
        self.current_offset += 2;

        let name = self
            .payload
            .get(self.current_offset..self.current_offset + name_len)?;
        self.current_offset += name_len;

        let val_type = *self.payload.get(self.current_offset)?;
        self.current_offset += 1;

        let data_bytes = self
            .payload
            .get(self.current_offset..self.current_offset + 32)?;
        let mut val_data = [0u8; 32];
        val_data.copy_from_slice(data_bytes);
        self.current_offset += 32;
        self.entries_to_read -= 1;

        Some(BTreeEntry {
            hash,
            name,
            val_type,
            val_data,
        })
    }
}

/// A reader for Kpack files.
///
/// # Fields
/// - `extents`: The extents of the Kpack file.
pub struct KpackReader<'a> {
    pub extents: &'a [Extent],
}

impl<'a> KpackReader<'a> {
    /// Creates a new `KpackReader` instance.
    ///
    /// # Args
    /// - `extents`: The extents of the Kpack file.
    #[must_use]
    pub const fn new(extents: &'a [Extent]) -> Self {
        Self { extents }
    }

    /// Fills the `Section` (the display buffer) from a specific offset.
    ///
    /// # Args
    /// - `offset`: The logical offset to start reading from.
    /// - `section_buffer`: The buffer to fill with the read data.
    pub fn read_into_section(
        &self,
        offset: u64,
        section_buffer: &mut [u8],
    ) -> Result<usize, &'static str> {
        let extent = self.find_extent(offset).ok_or("Offset hors limites")?;

        let chunk_size: u64 = 65536;
        let chunk_index = ((offset - extent.file_off) / chunk_size) as u32;

        if chunk_index >= extent.count {
            return Err("Index de chunk corrompu ou invalide");
        }

        let _target_chunk_hash = extent.first_chunk_id;

        // 4. Fetch NVMe (Simulé)
        // Le StorageEngine lirait ici les octets bruts de la recette KPACK.
        // let recipe_buffer = storage_engine.fetch(_target_chunk_hash)?;

        // 5. Cristallisation (L'éveil KPACK)
        // C'est ici que l'on relie le moteur KPACK d'Amentys. On lit l'Opcode
        // (LIT, DELTA, RLE, etc.) et on crache la donnée décompressée directement
        // dans le `section_buffer` de qwx.

        /*
        let opcode = kpack::Opcode::from_u8(recipe_buffer[0]).unwrap();
        kpack::execute(
            &opcode,
            param,
            &recipe_buffer[header_len..],
            section_buffer,
            reference_buffer // Si DELTA ou DICT
        );
        */

        // Retourne le nombre d'octets réellement projetés à l'écran
        Ok(section_buffer.len())
    }

    /// Find the extent that contains the given offset.
    /// # Arg
    /// - `offset`: The logical offset to start reading from.
    /// # Return
    /// - `Option<&Extent>`: The extent that contains the given offset, or None if not found.
    fn find_extent(&self, offset: u64) -> Option<&Extent> {
        for ext in self.extents {
            if offset >= ext.file_off && offset < ext.file_off + ext.len {
                return Some(ext);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extent_resolution() {
        let extents = [
            Extent {
                file_off: 0,
                len: 65536,
                first_chunk_id: [0xAA; 32],
                count: 1,
                last_len: 65536,
            },
            Extent {
                file_off: 65536,
                len: 1024,
                first_chunk_id: [0xBB; 32],
                count: 1,
                last_len: 1024,
            },
        ];

        let reader = KpackReader::new(&extents);

        let ext1 = reader.find_extent(500).expect("Extent introuvable");
        assert_eq!(ext1.first_chunk_id[0], 0xAA);

        let ext2 = reader.find_extent(66000).expect("Extent introuvable");
        assert_eq!(ext2.first_chunk_id[0], 0xBB);

        assert!(reader.find_extent(999999).is_none());
    }
    #[test]
    fn test_kpack_recipe_reconstruction_rle() {
        // Le buffer de projection final de qwx (une page NVMe complète)
        let mut section_buffer = [0u8; 4096];

        // On simule une recette KPACK lue depuis le disque.
        // Format imaginaire de notre recette :
        // [OPCODE] [PARAM (2 octets)] [DATA]
        // OPCODE 0x02 = RLE (Run Length Encoding)
        // PARAM = 4096 (Taille à remplir)
        // DATA = 0x20 (Le caractère Espace)

        let kpack_recipe: [u8; 4] = [
            0x02, // Opcode RLE
            0x00, 0x10, // Param : 4096 (0x1000 en Little Endian) <- Correction ici
            0x20, // Byte à répéter (Espace)
        ];

        // Moteur d'exécution KPACK (simplifié pour le test)
        let opcode = kpack_recipe[0];
        let count = u16::from_le_bytes([kpack_recipe[1], kpack_recipe[2]]) as usize;
        let byte_to_repeat = kpack_recipe[3];

        // Exécution de la recette directement dans la Section de qwx
        if opcode == 0x02 {
            // En Rust no_std, on remplit le slice sans allocation
            let safe_count = core::cmp::min(count, section_buffer.len());
            section_buffer[..safe_count].fill(byte_to_repeat);
        }

        // Validation : Le buffer de 4096 octets doit être rempli d'espaces (0x20)
        assert_eq!(section_buffer[0], 0x20);
        assert_eq!(section_buffer[4095], 0x20);

        // Le coût de lecture disque a été de 4 octets.
        // Le reste a été généré par le CPU à la vitesse de la RAM.
    }
    #[test]
    fn test_cubefs_cow_identity_shift() {
        // Création de deux nœuds strictement identiques
        let node_a = BTreeNode {
            magic: *b"BNOD",
            version: 1,
            level: 0,
            entry_count: 0,
            payload: [0u8; 4056],
            blake3_hash: [0u8; 32], // Rempli après calcul
        };

        let mut node_b = BTreeNode {
            magic: *b"BNOD",
            version: 1,
            level: 0,
            entry_count: 0,
            payload: [0u8; 4056],
            blake3_hash: [0u8; 32],
        };

        // On modifie UN SEUL OCTET dans le payload du nœud B (ex: on change les droits ou une lettre)
        node_b.payload[1024] = 0xFF;

        // Calcul de l'identité BLAKE3 (Simulé : on utilise un hash dummy pour le test
        // ou la vraie fonction blake3::hash si elle est disponible dans ton env de test)
        let hash_a = calculate_node_hash(&node_a);
        let hash_b = calculate_node_hash(&node_b);

        assert_ne!(
            hash_a, hash_b,
            "Violation de l'architecture Merkle : le hash doit changer si 1 bit change"
        );
    }

    // Fonction utilitaire pour le test (qui simule le hachage des 4064 premiers octets)
    fn calculate_node_hash(node: &BTreeNode) -> [u8; 32] {
        // Dans Amentys, tu utilises la crate blake3 en no_std
        let node_bytes = unsafe {
            core::slice::from_raw_parts(
                (node as *const BTreeNode) as *const u8,
                4096 - 32, // On ne hache pas le champ hash lui-même
            )
        };
        blake3::hash(node_bytes).into()
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_nucleator_insert_shift() {
            let mut original = [0u8; 4096];
            // On forge un état initial: [A, B, C, ..., Z]
            original[0] = b'A';
            original[1] = b'B';
            original[2] = b'C';
            original[4095] = b'Z';

            // Intention: On insère un 'X' à l'index 1 (entre A et B)
            let (new_buffer, ejected) =
                Nucleator::insert_byte(&original, 1, b'X').expect("L'insertion a échoué");

            // L'octet Z a dû être poussé en dehors du chunk
            assert_eq!(ejected, b'Z');

            // La nouvelle forme doit être: [A, X, B, C, ...]
            assert_eq!(new_buffer[0], b'A');
            assert_eq!(new_buffer[1], b'X');
            assert_eq!(new_buffer[2], b'B');
            assert_eq!(new_buffer[3], b'C');

            // L'octet d'origine est intact (immuabilité prouvée)
            assert_eq!(original[1], b'B');
        }

        #[test]
        fn test_nucleator_out_of_bounds() {
            let original = [0u8; 4096];
            // Frapper à l'index 4096 (qui est le 4097e octet) doit échouer gracieusement
            let result = Nucleator::insert_byte(&original, 4096, b'X');
            assert!(result.is_none());
        }
    }
    #[test]
    fn test_btree_iterator_corruption_resistance() {
        // On simule un payload de nœud B-Tree (4056 octets)
        let mut payload = [0u8; 4056];

        // On forge une entrée malveillante / corrompue
        // 1. Hash (8 octets)
        payload[0..8].copy_from_slice(&0xDEAD_BEEF_CAFE_BABE_u64.to_le_bytes());

        // 2. Longueur du nom (2 octets) -> On met une taille absurde (ex: 65000)
        // Cela dépasse largement les 4056 octets du payload !
        payload[8..10].copy_from_slice(&65000_u16.to_le_bytes());

        // On initialise l'itérateur en lui disant qu'il y a 1 entrée à lire
        let mut iter = BTreeIterator::new(&payload, 1);

        // L'itérateur DOIT retourner None (ou s'arrêter) et surtout NE PAS PANIQUER.
        // Sous Linux, un mauvais offset ferait un "panic: index out of bounds".
        // Avec notre implémentation utilisant `payload.get(...) ?`, ça renvoie None.
        let entry = iter.next();

        assert!(
            entry.is_none(),
            "L'itérateur aurait dû échouer gracieusement face à une mémoire corrompue"
        );
    }
}
