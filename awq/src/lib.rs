#![cfg_attr(not(test), no_std)]

use core::assert;
use core::clone::Clone;
use core::cmp::Eq;
use core::cmp::PartialEq;
use core::derive;
use core::fmt::Debug;
use core::iter::Iterator;
use core::result::Result;
use core::result::Result::Ok;
use heapless::Vec;
use noun::Noun;

/// The maximum number of simultaneous branches in memory for Amentys
pub const MAX_BRANCHES: usize = 16;
/// Represent the type of branch in the Amentys system.
///
/// # Note
/// - `Main`: The official and immutable state of the machine.
/// - `Ephemeral`: A temporary branch (e.g., sandbox for running a tool).
/// - `Deployment`: A network image being applied (via Kpack Delta) with a target Noun (the final image dictated by the server).
///
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BranchType {
    /// The official and immutable state of the machine.
    Main,
    /// A temporary branch (e.g., sandbox for running a tool).
    /// It will be destroyed by the Phoenix Sweeper when the timestamp is exceeded.
    Ephemeral { expiration_timestamp: u64 },

    /// Your brilliant idea: a network image being applied (via Kpack Delta).
    /// It has a target Noun (the final image dictated by the server).
    Deployment { target_noun: Noun },
}

/// Represents a tip in the Merkle tree of the filesystem.
///
/// # Fields
/// - `branch_type`: The type of the branch (Main, Ephemeral, Deployment).
/// - `root`: The Noun representing the root of the branch in the Merkle tree.
///
#[derive(Debug, Clone)]
pub struct Branch {
    pub branch_type: BranchType,
    pub root: Noun,
}

/// The state of the Amentys system, including all branches and the currently active main branch.
///
/// # Fields
///
/// - `branches`: A vector containing all branches (Main, Ephemeral, Deployment).
/// - `active_main_index`: The index of the currently active Main branch (the one the OS is running on).
///
pub struct AwqState {
    /// The list of all branches (Main, Ephemeral, Deployment)
    pub branches: Vec<Branch, MAX_BRANCHES>,
    /// The index of the currently active Main branch (the one the OS is running on)
    pub active_main_index: usize,
}
impl AwqState {
    /// Init the system with an initial root (the Noun from the boot disk)
    /// # Arguments
    /// * `root_noun` - The initial root Noun representing the state of the system at boot time.
    /// # Returns
    /// An instance of `AwqState` with the initial Main branch set to the provided root Noun.
    /// # Examples
    /// ```no_run
    /// let root_noun = Noun::of(&[0x00; 32]);
    /// let state = AwqState::new(&root_noun);
    /// ```
    /// # Panics
    /// This function will panic if the initial branch cannot be added to the `branches` vector, which should not happen under normal circumstances since the initial state is always valid.
    #[must_use]
    pub fn new(root_noun: &Noun) -> Self {
        let mut branches = Vec::new();
        let main_branch = Branch::new_main(root_noun);
        assert!(branches.push(main_branch).is_ok());
        Self {
            branches,
            active_main_index: 0,
        }
    }

    /// Retrieves the cryptographic root of the currently active system
    /// # Returns
    /// A reference to the Noun representing the root of the currently active Main branch.
    /// # Examples
    /// ```no_run
    /// use noun::Noun;
    /// use awq::AwqState;
    /// let root_noun = Noun::of(&[0x00; 32]);
    /// let state = AwqState::new(&root_noun);
    /// let active_root = state.get_active_root();
    /// ```
    #[must_use]
    pub fn get_active_root(&self) -> &Noun {
        &self.branches[self.active_main_index].root
    }

    /// Creates a disposable sandbox environment
    /// # Arguments
    /// * `expiration_timestamp` - The timestamp after which the ephemeral branch will be considered expired
    /// # Returns
    /// A `Result` indicating success or failure. On success, the ephemeral branch is added
    /// to the list of branches. On failure, an error message is returned if the maximum number of branches has been reached.
    /// # Errors
    /// Returns an error if the maximum number of branches has been reached.
    /// # Examples
    /// ```no_run
    /// use noun::Noun;
    /// use awq::AwqState;
    /// let root_noun = Noun::of(&[0x00; 32]);
    /// let mut state = AwqState::new(&root_noun);
    /// assert!(state.spawn_ephemeral(1234567890).is_ok());
    /// ```
    pub fn spawn_ephemeral(&mut self, expiration_timestamp: u64) -> Result<(), &'static str> {
        let root = self.get_active_root().clone();
        let branch = Branch::new_ephemeral(&root, expiration_timestamp);
        self.branches
            .push(branch)
            .map_err(|_| "Erreur: Limite de branches atteinte")
    }

    /// Prepares the computer to receive a network image (The Ghost 2.0)
    /// Returns the index of the new deployment branch.
    /// # Arguments
    /// * `target_noun` - The Noun representing the target state of the system
    /// # Returns
    /// A `Result` containing the index of the new deployment branch on success, or an error message if the maximum number of branches has been reached.
    /// # Errors
    /// Returns an error if the maximum number of branches has been reached.
    /// # Examples
    /// ```no_run
    /// let mut state = AwqState::new(&root_noun);
    /// let target_noun = Noun::of(&[0xFF; 32]);
    /// let index = state.spawn_deployment(target_noun).expect("Failed to spawn deployment");
    /// ```
    pub fn spawn_deployment(&mut self, target_noun: &Noun) -> Result<usize, &'static str> {
        let root = self.get_active_root().clone();
        let branch = Branch::new_deployment(&root, target_noun);
        self.branches
            .push(branch)
            .map_err(|_| "Erreur: Limite de branches atteinte")?;
        Ok(self.branches.len() - 1)
    }
    pub fn spawn_ephemeral_from(
        &mut self,
        root_noun: &Noun,
        expiration_timestamp: u64,
    ) -> Result<(), &'static str> {
        let branch = Branch::new_ephemeral(root_noun, expiration_timestamp);
        self.branches
            .push(branch)
            .map_err(|_| "Erreur: Limite de branches atteinte")
    }
    /// Attempts to permanently switch to the new network system.
    /// If the Kpack signature is bad, the system remains on the old OS without flinching.
    /// # Arguments
    /// * `index` - The index of the deployment branch to finalize
    /// # Returns
    /// Returns `true` if the deployment was successfully finalized and the system switched to the new main branch, or `false` if the index is invalid or the promotion failed.
    /// # Examples
    /// ```no_run
    /// use noun::Noun;
    /// use awq::AwqState;
    /// let root_noun = Noun::of(&[0x00; 32]);
    /// let mut state = AwqState::new(&root_noun);
    /// let target_noun = Noun::of(&[0xFF; 32]);
    /// let index = state.spawn_deployment(target_noun).expect("Failed to spawn deployment");
    /// let success = state.finalize_deployment(index);
    /// ```
    pub fn finalize_deployment(&mut self, index: usize) -> bool {
        if index >= self.branches.len() {
            return false;
        }

        if self.branches[index].promote_deployment() {
            self.active_main_index = index;
            return true;
        }
        false
    }

    /// Iterates through the branches, destroys those that have expired, and reorganizes memory.
    /// # Arguments
    /// * `current_timestamp` - The current timestamp used to determine which ephemeral branches have expired
    /// # Examples
    /// ```no_run
    /// use noun::Noun;
    /// use awq::AwqState;
    /// let root_noun = Noun::of(&[0x00; 32]);
    /// let mut state = AwqState::new(&root_noun);
    /// state.sweep(current_timestamp);
    /// ```
    /// # Note
    /// This function modifies the internal state of `AwqState` by removing expired ephemeral branches and updating the index of the active main branch if necessary.
    pub fn sweep(&mut self, current_timestamp: u64) {
        let mut new_branches = Vec::new();
        let mut new_active_index = 0;

        for (i, branch) in self.branches.iter().enumerate() {
            // On vérifie si la branche est morte
            let keep = if let BranchType::Ephemeral {
                expiration_timestamp,
            } = branch.branch_type
                && current_timestamp >= expiration_timestamp
            {
                false
            } else {
                true
            };

            if keep {
                // Si c'était notre Main active, on met à jour son nouvel index
                if i == self.active_main_index {
                    new_active_index = new_branches.len();
                }
                let _ = new_branches.push(branch.clone());
            }
        }
        self.branches = new_branches;
        self.active_main_index = new_active_index;
    }
}

impl Branch {
    /// Starts the normal system (the OS boots from this branch)
    /// # Arguments
    /// * `root` - The Noun representing the root of the main branch
    /// # Returns
    /// An instance of `Branch` representing the main branch of the system.
    /// # Examples
    /// ```no_run
    /// use noun::Noun;
    /// use awq::Branch;
    /// let root_noun = Noun::of(&[0x00; 32]);
    /// let main_branch = Branch::new_main(&root_noun);
    /// ```
    #[must_use]
    pub fn new_main(root: &Noun) -> Self {
        Self {
            branch_type: BranchType::Main,
            root: root.clone(),
        }
    }

    /// Opens a secure sandbox (e.g., for running a tool). It will be destroyed by the Phoenix Sweeper when the timestamp is exceeded.
    /// # Arguments
    /// * `root` - The Noun representing the root of the ephemeral branch
    /// * `expiration` - The timestamp after which the ephemeral branch will be considered expired
    /// # Returns
    /// An instance of `Branch` representing the ephemeral branch.
    /// # Examples
    /// ```no_run
    /// use noun::Noun;
    /// use awq::Branch;
    /// let root_noun = Noun::of(&[0x00; 32]);
    /// let ephemeral_branch = Branch::new_ephemeral(&root_noun, expiration_timestamp);
    /// ```
    #[must_use]
    pub fn new_ephemeral(root: &Noun, expiration: u64) -> Self {
        Self {
            branch_type: BranchType::Ephemeral {
                expiration_timestamp: expiration,
            },
            root: root.clone(), // Au départ, elle est un clone exact de la racine d'où elle part
        }
    }

    /// Prepares a network restoration (PXE/Ghost 2.0)
    /// # Arguments
    /// * `current_root` - The Noun representing the current root of the system
    /// * `target` - The Noun representing the target state of the system to be achieved through deployment
    /// # Returns
    /// An instance of `Branch` representing the deployment branch.
    /// # Examples
    /// ```no_run
    /// use noun::Noun;
    /// use awq::Branch;
    /// let current_root_noun = Noun::of(&[0x00; 32]);
    /// let target_noun = Noun::of(&[0x00; 32]);
    /// let deployment_branch = Branch::new_deployment(&current_root_noun, &target_noun);
    /// ```
    #[must_use]
    pub fn new_deployment(current_root: &Noun, target: &Noun) -> Self {
        Self {
            branch_type: BranchType::Deployment {
                target_noun: target.clone(),
            },
            root: current_root.clone(),
        }
    }

    /// The magic of network deployment:
    /// Attempts to promote the deployment branch to the main system.
    /// Fails if the Kpack patches did not achieve the exact Noun from the server.
    /// # Returns
    /// `true` if the promotion was successful and the branch type was changed to Main, or `false` if the promotion failed (e.g., the current root does not match the target Noun).
    /// # Examples
    /// ```no_run
    /// use noun::Noun;
    /// use awq::Branch;
    /// let current_root_noun = Noun::of(&[0x00; 32]);
    /// let target_noun = Noun::of(&[0xFF; 32]);
    /// let mut deployment_branch = Branch::new_deployment(&current_root_noun, &target_noun);
    /// deployment_branch.root = target_noun;
    /// let success = deployment_branch.promote_deployment();
    /// ```
    /// # Note
    /// This function modifies the internal state of the `Branch` by changing its type to Main if the promotion is successful. It ensures that the deployment branch can only be promoted if the current root matches the target Noun, providing a security guarantee against tampering or incomplete updates.
    /// # Safety
    /// This function assumes that the `Branch` is in a valid state and that the `root` and `target_noun` are correctly set. It does not perform any additional validation beyond
    /// checking for equality between the current root and the target Noun.
    /// # Panics
    /// This function does not panic under normal circumstances. However, if the `Branch` is
    /// not in a Deployment state when called, it will simply return `false` without making any changes.
    /// # See Also
    /// - [`AwqState::finalize_deployment`] for how this function is typically used in the context of the overall system state management.
    /// # Returns
    /// - `true` if the promotion was successful and the branch type was changed to Main
    /// - `false` if the promotion failed (e.g., the current root does not match the target Noun or the branch is not a Deployment)
    pub fn promote_deployment(&mut self) -> bool {
        if let BranchType::Deployment { target_noun } = &self.branch_type {
            // Absolute security: the network clone is only validated if the hash
            // calculated on the disk is STRICTLY identical to that of the server.
            if self.root == *target_noun {
                self.branch_type = BranchType::Main;
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::assert;
    use core::assert_eq;
    use core::prelude::rust_2024::test;
    #[test]
    fn test_awq_state_deployment_lifecycle() {
        let initial_noun = Noun::of(&[0x01; 32]);
        let mut state = AwqState::new(&initial_noun);

        let server_target = Noun::of(&[0xFF; 32]); // La cible du réseau

        let deploy_index = state.spawn_deployment(&server_target).unwrap();
        assert_eq!(state.branches.len(), 2);

        state.branches[deploy_index].root = server_target;

        let success = state.finalize_deployment(deploy_index);

        assert!(success);
        assert_eq!(state.active_main_index, deploy_index); // L'OS a basculé !
    }

    #[test]
    fn test_phoenix_sweeper() {
        let initial_noun = Noun::of(&[0x01; 32]);
        let mut state = AwqState::new(&initial_noun);

        // On crée une branche qui expire à T=100
        state.spawn_ephemeral(100).unwrap();
        // On crée une branche qui expire à T=200
        state.spawn_ephemeral(200).unwrap();

        assert_eq!(state.branches.len(), 3);

        // Le temps passe... On est à T=150
        state.sweep(150);

        // La première branche doit avoir été incinérée par le Phoenix Sweeper
        assert_eq!(state.branches.len(), 2);

        // On vérifie que la branche survivante est bien celle de T=200
        if let BranchType::Ephemeral {
            expiration_timestamp,
        } = state.branches[1].branch_type
        {
            assert_eq!(expiration_timestamp, 200);
        } else {
            panic!("La mauvaise branche a été supprimée !");
        }
    }
    #[test]
    fn test_deployment_promotion_success() {
        let server_target = Noun::of(&[0x42; 32]);
        let mut deployment = Branch::new_deployment(&Noun::of(&[0; 32]), &server_target);

        deployment.root = server_target; // Objectif atteint
        assert!(deployment.promote_deployment());
        assert_eq!(deployment.branch_type, BranchType::Main);
    }

    #[test]
    fn test_deployment_promotion_failure() {
        let server_target = Noun::of(&[0x42; 32]);
        let mut deployment = Branch::new_deployment(&Noun::of(&[0; 32]), &server_target);

        deployment.root = Noun::of(&[0x11; 32]);
        assert!(!deployment.promote_deployment());
    }
}
