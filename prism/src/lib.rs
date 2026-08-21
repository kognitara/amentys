#![cfg_attr(not(test), no_std)]

use awq::AwqState;
use jinshu::router::SemanticRouter;
use jinshu::storage::BlockDevice;
use plan::Plan;
use plan::layer::Capabilities;
use ra::fs::nvme::allocator::Allocator;

pub struct Prism {
    pub plan: Plan,
}

impl Prism {
    #[must_use]
    pub const fn new(plan: Plan) -> Self {
        Self { plan }
    }

    /// Exécute le Prisme. Retourne une erreur si l'environnement ne peut pas être monté.
    pub fn run<T: BlockDevice>(
        &mut self,
        router: &mut SemanticRouter<'_, T>,
    ) -> Result<(), &'static str> {
        // CORRECTION : On renvoie des Err() au lieu de paniquer
        if self.plan.directory_is_null() {
            return Err("Directory Noun cannot be null");
        } else if self.plan.phoenix_is_empty() {
            return Err("Phoenix Layers cannot be empty");
        }

        if self.plan.effective_capabilities() == Capabilities::None {
            return Ok(()); // Le plan n'a pas de droits, on quitte silencieusement et proprement
        }

        let directory = self.plan.get_directory();
        let mut awq = AwqState::new(&directory);

        let mut allocator = Allocator::new(0x0100_0000, 0x0200_0000);
        let mut final_app_noun = directory;

        for layer in self.plan.get_layers() {
            // CORRECTION : ok_or(...)? au lieu de expect(...)
            let phys_addr = allocator
                .allocate_page()
                .ok_or("OOM: Pas assez de RAM physique pour la fusion")?;

            let virt_addr = phys_addr;

            // CORRECTION : L'opérateur ? propage directement l'erreur de merge_trees
            final_app_noun =
                router.merge_trees(&final_app_noun, &layer.root, phys_addr, virt_addr)?;
        }

        // CORRECTION : map_err au lieu de unwrap() pour Awq
        awq.spawn_ephemeral_from(&final_app_noun, u64::MAX)
            .map_err(|_| "Impossible de créer la branche éphémère dans Awq")?;

        loop {
            if self.plan.should_quit() {
                self.plan.clear_layers();
                awq.sweep(u64::MAX);
                break;
            }
            // Simulation de la boucle de l'interpréteur
            core::hint::spin_loop();
        }
        Ok(())
    }
}
