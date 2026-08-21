#![no_std]

use chacha20::cipher::{KeyIvInit, StreamCipher};
use chacha20::{ChaCha20, Key, Nonce};
use core::arch::x86_64::_rdtsc;
use core::sync::atomic::{AtomicU64, Ordering};
use rand_chacha::ChaCha20Rng;
use rand_core::{SeedableRng, TryRng};
use x25519_dalek::{EphemeralSecret, PublicKey};

/// Global atomic counter for network interrupts
pub static NETWORK_IRQ_COUNTER: AtomicU64 = AtomicU64::new(0);

/// The `Zuu` struct is responsible for generating a steganographic faux noun
/// and determining if the system is under attack based on network pressure.
///
/// # Fields
/// * `pressure_threshold` - A threshold value for network pressure detection.
pub struct Zuu {
    pressure_threshold: u64,
}

impl Zuu {
    /// create a new instance of Zuu with a specified pressure threshold
    ///
    /// # Arguments
    /// * `threshold` - The threshold value for network pressure detection
    ///
    /// # Returns
    /// * `Zuu` - A new instance of Zuu with the specified pressure threshold
    ///
    #[must_use]
    pub const fn new(threshold: u64) -> Self {
        Self {
            pressure_threshold: threshold,
        }
    }

    /// Check if the system is under attack based on the network interrupt counter
    ///
    /// # Returns
    /// * `bool` - Returns true if the system is under attack, false otherwise
    pub fn is_under_attack(&self) -> bool {
        // Aspiration atomique et remise à zéro
        let irq_burst = NETWORK_IRQ_COUNTER.swap(0, Ordering::Relaxed);
        irq_burst > self.pressure_threshold
    }

    /// Gather hardware entropy using the RDTSC instruction
    ///
    /// # Returns
    /// * `u64` - A 64-bit value representing the gathered hardware entropy
    fn gather_hardware_entropy() -> u64 {
        // SAFETY: _rdtsc lit le compteur de cycles en Ring 0 sans altérer la mémoire.
        let tsc = unsafe { _rdtsc() };
        tsc ^ 0xCAFE_BABE_DEAD_BEEFu64
    }

    /// Generate and encrypt the Steganographic Faux Noun
    ///
    /// # Parameters
    /// * `pulse_public_key_bytes` - The public key of Pulse
    /// * `pulse_bitmask` - The bitmask used for steganographic encoding
    /// # Returns
    /// * `([u8; 32], [u8; 32])` - A tuple containing Zuu's public key and the encrypted noun
    /// # Panics
    /// This function will panic if the random number generator fails to fill the `raw_noun` with entropy.
    #[must_use]
    pub fn craft_steganographic_noun(
        &self,
        pulse_public_key_bytes: &[u8; 32],
        pulse_bitmask: u64,
    ) -> ([u8; 32], [u8; 32]) {
        let mut raw_noun = [0_u8; 32];
        let mut rng = ChaCha20Rng::seed_from_u64(Self::gather_hardware_entropy());

        // 1. Construction du bruit visuel (remplissage stéganographique aléatoire)
        rng.try_fill_bytes(&mut raw_noun)
            .expect("failed to fill raw_noun with entropy");

        // 2. Détermination de l'ordre (Kpack inversé)
        let is_attack = self.is_under_attack();
        let command_byte: u8 = if is_attack { 0xFF } else { 0x00 };

        // 3. Injection furtive de l'ordre masqué dans le premier octet (nuance de couleur)
        raw_noun[0] =
            command_byte ^ u8::try_from(pulse_bitmask).expect("pulse_bitmask out of range");

        // 4. ECDH : Zuu forge sa propre clé asymétrique éphémère
        let zuu_secret = EphemeralSecret::random_from_rng(&mut rng);
        let zuu_public = PublicKey::from(&zuu_secret);

        // 5. ECDH : Calcul du secret partagé avec la clé de Pulse
        let pulse_pub = PublicKey::from(*pulse_public_key_bytes);
        let shared_secret = zuu_secret.diffie_hellman(&pulse_pub);

        // 6. Chiffrement de Flux In-Place (ChaCha20)
        let key = Key::from(shared_secret.to_bytes());
        // Un nonce fixe à 0 est sûr ici car le `shared_secret` change à chaque
        // exécution (grâce aux clés éphémères de Pulse et Zuu).
        let nonce = Nonce::from([0_u8; 12]);

        let mut cipher = ChaCha20::new(&key, &nonce);

        // Les couleurs sont cryptographiquement brouillées, sans ajout d'allocations
        cipher.apply_keystream(&mut raw_noun);

        // Zuu retourne son empreinte et le calque chiffré.
        (zuu_public.to_bytes(), raw_noun)
    }
}
