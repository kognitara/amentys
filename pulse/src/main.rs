#![cfg_attr(not(test), no_main, no_std)]

use core::arch::x86_64::_rdtsc;
use core::hint::spin_loop;
use core::panic::PanicInfo;
use pulse::Pulse;
use x86_64::instructions::hlt;

// Nos "Plateaux d'argent" en mémoire partagée
static mut ZUU_PUBLIC_KEY: [u8; 32] = [0; 32];
static mut JI_PRIVATE_KEY: [u8; 32] = [0; 32];

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut pulse = Pulse::new();

    loop {
        // 1. Le cœur bat : génération des clés asymétriques et du bitmask
        pulse.beat();

        // 2. La Distribution sur les plateaux d'argent
        // SAFETY: Nous sommes dans la boucle principale Ring 0, accès exclusif.
        unsafe {
            // SAFETY: Nous sommes dans la boucle principale Ring 0, accès exclusif.
            ZUU_PUBLIC_KEY = pulse.current_public_key;
            JI_PRIVATE_KEY = pulse.get_private_key_for_ji();
        }

        // 3. L'Arythmie : Calcul du délai aléatoire avant le prochain battement
        // On utilise le current_bitmask pour déduire un nombre de cycles aléatoire
        let delay_cycles = 10_000_000 + (pulse.current_bitmask % 40_000_000);

        let start_time = unsafe {
            // SAFETY: L'instruction _rdtsc est sûre à appeler sur l'architecture x86_64
            // à partir du Ring 0. Elle lit le Time Stamp Counter directement depuis
            // le processeur sans modifier ou corrompre l'état de la mémoire.
            _rdtsc()
        };

        // 4. L'Attente furtive
        while unsafe {
            // SAFETY: L'instruction _rdtsc est sûre à appeler sur l'architecture x86_64
            _rdtsc()
        } < start_time + delay_cycles
        {
            // spin_loop indique au CPU de consommer moins d'énergie en attendant
            spin_loop();
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        hlt();
    }
}
