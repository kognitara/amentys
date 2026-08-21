#![cfg_attr(not(test), no_std)]

#[doc = "The `fs` module contains the implementation of the file system."]
pub mod fs;
#[doc = "The `tree` module contains the implementation of the Trie data structure."]
pub mod tree;

/// The `RaHealth` enum represents the health status of the Ra (Application Kernel).
///
/// # Variants
/// * `Ok` - The system is healthy and all plans are running normally.
/// * `PlanFailed(usize)` - A specific plan has failed, returning its ID.
/// * `Compromised` - The application kernel has detected a compromise or corruption.
pub enum RaHealth {
    /// The system is healthy and all plans are running normally.
    Ok,
    /// A specific plan has failed, returning its ID.
    PlanFailed(usize),
    /// The application kernel has detected a compromise or corruption.
    Compromised,
}

/// The `PlanState` enum represents the state of a plan (process or application) in the Ra (Application Kernel).
/// # Variants
/// * `Empty` - The plan slot is free in RAM.
/// * `Running` - The plan is currently running.
/// * `Failed` - The plan has crashed (e.g., Segfault).
#[derive(Copy, Clone)]
pub enum PlanState {
    /// The plan slot is free in RAM.
    Empty,
    /// The plan is currently running.
    Running,
    /// The plan has crashed (e.g., Segfault).
    Failed,
}

/// The `Plan` struct represents a plan (process or application) managed by the Ra (Application Kernel).
/// # Fields
/// * `id` - The unique identifier of the plan.
/// * `state` - The current state of the plan, represented by the `PlanState` enum.
#[derive(Copy, Clone)]
pub struct Plan {
    pub id: usize,
    pub state: PlanState,
}

/// The Ra (Application Kernel) manages the lifecycle of plans (processes or applications).
pub struct Ra {
    pub plans: [Plan; 64],
}

impl Default for Ra {
    fn default() -> Self {
        Self::new()
    }
}

impl Ra {
    /// Create a new instance of Ra with an empty plan list.
    /// # Returns
    /// * `Ra` - A new instance of Ra with an empty plan list
    #[must_use]
    pub const fn new() -> Self {
        const EMPTY_PLAN: Plan = Plan {
            id: 0,
            state: PlanState::Empty,
        };

        Self {
            plans: [EMPTY_PLAN; 64],
        }
    }

    /// The application management cycle. Called in a loop by `re`.
    pub fn tick(&mut self) -> RaHealth {
        for plan in &mut self.plans {
            match plan.state {
                PlanState::Failed => {
                    return RaHealth::PlanFailed(plan.id);
                }
                PlanState::Running | PlanState::Empty => {}
            }
        }
        RaHealth::Ok
    }
}
