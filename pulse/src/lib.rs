#![no_std]

use core::arch::x86_64::_rdtsc;
use core::ptr::write_volatile;
use rand_chacha::ChaCha20Rng;
use rand_core::SeedableRng;
use x25519_dalek::{EphemeralSecret, PublicKey};

pub struct Pulse {
    /// La clé publique actuelle (à distribuer à Zuu)
    pub current_public_key: [u8; 32],

    /// La clé privée (strictement réservée à Ji)
    current_private_key: [u8; 32],

    /// Le masque binaire (pour extraire l'ordre du Faux Noun)
    pub current_bitmask: u64,
}

impl Default for Pulse {
    fn default() -> Self {
        Self::new()
    }
}

impl Pulse {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            current_public_key: [0; 32],
            current_private_key: [0; 32],
            current_bitmask: 0,
        }
    }

    /// Extrait le chaos matériel pour générer une graine
    fn gather_hardware_entropy() -> u64 {
        // SAFETY: L'instruction _rdtsc est sûre à appeler sur l'architecture x86_64
        // à partir du Ring 0. Elle lit le Time Stamp Counter directement depuis
        // le processeur sans modifier ou corrompre l'état de la mémoire.
        let tsc = unsafe { _rdtsc() };

        // Mélange bit à bit pour lisser l'entropie matérielle
        tsc ^ 0xDEAD_BEEF_CAFE_BABEu64
    }

    /// Le Battement : Invoqué par le timer matériel (ex: APIC)
    pub fn beat(&mut self) {
        // 1. Incinération de l'ancienne clé privée
        self.burn_private_key();

        // 2. Génération de la nouvelle entropie
        let entropy = Self::gather_hardware_entropy();

        // 3. Génération des nouvelles clés asymétriques
        let (new_priv, new_pub) = Self::generate_curve25519_keypair(entropy);

        self.current_private_key = new_priv;
        self.current_public_key = new_pub;

        // 4. Génération du nouveau masque binaire pour l'opération AND
        self.current_bitmask = entropy.rotate_left(13);
    }

    /// Destruction chirurgicale de la clé privée en RAM
    fn burn_private_key(&mut self) {
        let ptr = self.current_private_key.as_mut_ptr();

        // SAFETY: `ptr` est un pointeur valide et aligné pointant vers le tableau
        // de la clé privée de cette instance exacte. write_volatile force le CPU
        // à écraser la RAM immédiatement avec des zéros, garantissant l'effacement
        // physique pour contrer les attaques de type Cold Boot.
        unsafe {
            for i in 0..32 {
                write_volatile(ptr.add(i), 0_u8);
            }
        }
    }

    fn generate_curve25519_keypair(seed: u64) -> ([u8; 32], [u8; 32]) {
        // 1. Initialiser le générateur CSPRNG avec notre entropie matérielle
        let mut rng = ChaCha20Rng::seed_from_u64(seed);

        // 2. Forger la clé privée éphémère
        let secret = EphemeralSecret::random_from_rng(&mut rng);

        // 3. Dériver la clé publique mathématiquement liée
        let public = PublicKey::from(&secret);

        // 4. Retourner les tableaux de 32 octets purs
        (secret.diffie_hellman(&public).to_bytes(), public.to_bytes())
    }
    /// Accès exclusif pour la lecture par Ji
    #[must_use]
    pub const fn get_private_key_for_ji(&self) -> [u8; 32] {
        self.current_private_key
    }
}
