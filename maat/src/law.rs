use plan::Plan;
use plan::sceau::{Sceau, Verdict};

/// Weighs a plan against a sceau and returns the verdict.
///
/// # Arguments
///
/// * `sceau` - The sceau to weigh against the plan.
/// * `plan` - The plan to be weighed.
///
/// # Returns
///
/// * `Verdict` - The verdict of the weighing.
#[must_use]
pub fn weigh(sceau: &Sceau, plan: &Plan) -> Verdict {
    sceau.weigh(plan)
}
