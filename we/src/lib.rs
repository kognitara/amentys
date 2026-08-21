#![no_std] // <-- On assume le no_std !

use heapless::String;

/// Représente un instantané immuable de notre code.
#[derive(Debug, Clone, PartialEq)]
pub struct Commit {
    /// Le hachage Blake3 du commit parent
    pub parent_hash: Option<[u8; 32]>,
    /// Le hachage Blake3 de l'arborescence (fichiers/dossiers)
    pub tree_hash: [u8; 32],
    /// Le message de validation (limité à 256 octets pour rester sur la stack)
    pub message: String<256>, 
    /// Le timestamp UNIX
    pub timestamp: u64,
}

impl Commit {
    pub fn new(tree_hash: [u8; 32], msg: &str, timestamp: u64) -> Self {
        // On convertit le &str en heapless::String (tronqué si ça dépasse 256 caractères)
        let mut message = String::new();
        let _ = message.push_str(msg); // Ignore silencieusement si c'est trop long

        Self {
            parent_hash: None,
            tree_hash,
            message,
            timestamp,
        }
    }

    pub fn set_parent(&mut self, parent: [u8; 32]) {
        self.parent_hash = Some(parent);
    }
}