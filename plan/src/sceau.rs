use crate::Plan;
use crate::layer::Capabilities;
use noun::Noun;

/// Seal of a Plan. Contains the root, effective capabilities, and layer count of the Plan.
///
/// # Fields
/// * `root` - The root of the Plan.
/// * `caps` - The effective capabilities of the Plan.
/// * `budget` - The budget for the Plan.
/// * `layers` - The number of layers in the Plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sceau {
    pub root: Noun,
    pub caps: Capabilities,
    pub budget: u32,
    pub layers: u8,
}

/// Verdict of weighing a Plan against a Sceau.
///
/// # Fields
/// * `Accept` - The Plan is compatible with the Sceau.
/// * `Refuse(&'static str)` - The Plan is not compatible with the Sceau, with a reason for refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Accept,
    Refuse(&'static str),
}

impl Sceau {
    /// Create a birth seal from a Plan. The seal contains the root, effective capabilities, and layer count of the Plan.
    ///
    /// # Arguments
    /// * `plan` - The Plan to create a seal from.
    /// * `budget` - The budget for the Plan.
    ///
    /// # Returns
    /// A new `Sceau` containing the root, effective capabilities, and layer count of the `Plan`.
    ///
    /// # Panics
    /// Panics if the `Plan`'s layer count exceeds `MAX_LAYERS`.
    #[must_use]
    pub fn birth(plan: &Plan, budget: u32) -> Self {
        let n = plan.layers.len();
        Self {
            root: plan.seal(),
            caps: plan.effective_capabilities(),
            budget,
            layers: if n > 255 {
                255
            } else {
                u8::try_from(n).expect("layer count fits in u8")
            },
        }
    }

    /// Weigh a `Plan` against the seal. If the `Plan` is not compatible with the seal, return a refusal verdict.
    ///
    /// The Plan is compatible if:
    /// - The `Plan`'s root matches the seal's root.
    /// - The `Plan`'s layer count does not exceed the seal's layer count.
    /// - The `Plan`'s effective capabilities do not exceed the seal's capabilities.
    ///
    /// # Returns
    /// If the `Plan` is compatible, return an acceptance verdict.
    ///
    /// # Panics
    /// Panics if the `Plan`'s layer count exceeds `MAX_LAYERS`.
    ///
    #[must_use]
    pub fn weigh(&self, plan: &Plan) -> Verdict {
        if plan.directory_is_null() {
            return Verdict::Refuse("null directory");
        }
        let sealed = plan.seal();
        if sealed != self.root {
            return Verdict::Refuse("noun mismatch");
        }
        if u8::try_from(plan.layers.len()).expect("layer count fits in u8") != self.layers {
            return Verdict::Refuse("layer count");
        }
        if !caps_allowed(&plan.effective_capabilities(), &self.caps) {
            return Verdict::Refuse("caps exceed seal");
        }
        Verdict::Accept
    }
}

/// Rank capabilities for comparison. Higher rank means more capabilities.
///
/// # Arguments
/// * `c` - The capabilities to rank.
///
/// # Returns
/// The rank of the capabilities, where higher numbers indicate more capabilities.
const fn cap_rank(c: &Capabilities) -> u8 {
    match c {
        Capabilities::None => 0,
        Capabilities::Read | Capabilities::Write | Capabilities::Execute => 1,
        Capabilities::ReadWrite | Capabilities::ReadExecute | Capabilities::WriteExecute => 2,
        Capabilities::ReadWriteExecute => 3,
        Capabilities::All => 4,
    }
}

/// Check if the capabilities of a `Plan` are allowed by the seal's capabilities.
///
/// # Arguments
/// * `have` - The capabilities of the `Plan`.
/// * `max` - The capabilities allowed by the seal.
///
/// # Returns
/// `true` if the `Plan`'s capabilities are allowed by the seal, `false` otherwise.
const fn caps_allowed(have: &Capabilities, max: &Capabilities) -> bool {
    cap_rank(have) <= cap_rank(max)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::{Layer, Layers};
    use noun::Noun;

    #[test]
    fn good_seal_accepted_bad_seal_refused() {
        let dir = Noun::of(b"root");
        let mut phoenix = Layers::new(dir.clone());
        phoenix
            .add_layer(Layer {
                name: "gamma",
                version: 1,
                description: "",
                root: Noun::of(b"g"),
                capabilities: Capabilities::Read,
            })
            .unwrap();
        let mut plan = Plan::new(dir, &mut phoenix, 0).unwrap();
        plan.add_layer(Layer {
            name: "tui",
            version: 1,
            description: "",
            root: Noun::of(b"tui"),
            capabilities: Capabilities::Read,
        })
        .unwrap();
        let sceau = Sceau::birth(&plan, 100);
        assert_eq!(sceau.weigh(&plan), Verdict::Accept);
        plan.add_layer(Layer {
            name: "net",
            version: 1,
            description: "",
            root: Noun::of(b"net"),
            capabilities: Capabilities::All,
        })
        .unwrap();
        assert_eq!(sceau.weigh(&plan), Verdict::Refuse("noun mismatch"));
    }
}
