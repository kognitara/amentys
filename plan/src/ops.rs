//! Seal + merge for `Plan`.
//!
//! Drop into `plan/src/ops.rs` then add to `plan/src/lib.rs`:
//! ```ignore
//! pub mod ops;
//! ```
//! `impl Plan` here is valid: every field of `Plan` is already `pub`.

use crate::Plan;
use crate::layer::{Capabilities, Layer, MAX_LAYERS};
use heapless::Vec;
use noun::Noun;

const fn cap_tag(c: &Capabilities) -> u8 {
    match c {
        Capabilities::None => 0,
        Capabilities::Read => 1,
        Capabilities::Write => 2,
        Capabilities::Execute => 3,
        Capabilities::ReadWrite => 4,
        Capabilities::ReadExecute => 5,
        Capabilities::WriteExecute => 6,
        Capabilities::ReadWriteExecute => 7,
        Capabilities::All => 8,
    }
}

fn absorb(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    hasher.update(bytes);
}

fn absorb_layer(hasher: &mut blake3::Hasher, layer: &Layer) {
    absorb(hasher, layer.name.as_bytes());
    absorb(hasher, &layer.version.to_le_bytes());
    absorb(hasher, layer.description.as_bytes());
    absorb(hasher, layer.root.as_bytes());
    absorb(hasher, &[cap_tag(&layer.capabilities)]);
}

impl Plan {
    /// Content-address of this Plan: directory + branch + layers + caps.
    /// Same content ⇒ same Noun, across boots.
    ///
    /// # Returns
    /// The Noun representing the seal of the Plan.
    ///
    /// # Panics
    /// Panics if the number of layers exceeds `MAX_LAYERS`.
    #[must_use]
    pub fn seal(&self) -> Noun {
        let mut hasher = blake3::Hasher::new();
        absorb(&mut hasher, self.directory.as_bytes());
        absorb(&mut hasher, &[self.branch_index]);
        absorb(&mut hasher, &[cap_tag(&self.effective_capabilities())]);
        absorb(
            &mut hasher,
            &u32::try_from(self.layers.len())
                .expect("Too many layers")
                .to_le_bytes(),
        );
        for layer in &self.layers {
            absorb_layer(&mut hasher, layer);
        }
        let hash = hasher.finalize();
        Noun::from_bytes(hash.as_bytes()).unwrap_or_else(Noun::null)
    }

    /// Deterministic merge. Layers are keyed by `root` Noun; first-seen wins.
    /// The result is a new Plan whose directory is the seal of the union.
    pub fn merge_with(&self, other: &Self) -> Result<Self, &'static str> {
        let mut layers: Vec<Layer, MAX_LAYERS> = Vec::new();

        for src in [&self.layers, &other.layers] {
            for layer in src {
                if layers.iter().any(|l| l.root == layer.root) {
                    continue;
                }
                layers
                    .push(layer.clone())
                    .map_err(|_| "merge: MAX_LAYERS")?;
            }
        }

        // stable order: by root bytes, then name
        let n = layers.len();
        for i in 0..n {
            for j in 0..n.saturating_sub(1).saturating_sub(i) {
                let swap = {
                    let a = &layers[j];
                    let b = &layers[j + 1];
                    a.root.as_bytes() > b.root.as_bytes()
                        || (a.root.as_bytes() == b.root.as_bytes() && a.name > b.name)
                };
                if swap {
                    layers.swap(j, j + 1);
                }
            }
        }

        let mut merged = Self {
            branch_index: self.branch_index.min(other.branch_index),
            directory: Noun::null(),
            layers,
            should_quit: false,
            layer_face: self.layer_face.clone(),
            phenix: self.phenix.clone(),
        };
        merged.directory = merged.seal();
        Ok(merged)
    }

    /// Clone Amentys: same layers, new sealed directory (identity of *this* copy).
    #[must_use]
    pub fn fork(&self) -> Self {
        let mut forked = self.clone();
        forked.should_quit = false;
        forked.directory = forked.seal();
        forked
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::Layers;

    fn layer(name: &'static str, root: Noun) -> Layer {
        Layer {
            name,
            version: 1,
            description: "",
            root,
            capabilities: Capabilities::Read,
        }
    }

    #[test]
    fn seal_is_stable_and_content_addressed() {
        let dir = Noun::of(b"dir");
        let mut phoenix = Layers::new(dir.clone());
        phoenix.add_layer(layer("gamma", Noun::of(b"g"))).unwrap();
        let mut a = Plan::new(dir.clone(), &mut phoenix, 0).expect("plan");
        a.add_layer(layer("alpha", Noun::of(b"a"))).unwrap();
        let s1 = a.seal();
        let s2 = a.seal();
        assert_eq!(s1, s2);
        a.add_layer(layer("beta", Noun::of(b"b"))).unwrap();
        assert_ne!(a.seal(), s1);
    }

    #[test]
    fn merge_is_commutative() {
        let dir = Noun::of(b"dir");
        let mut p1 = Layers::new(dir.clone());
        p1.add_layer(layer("gamma", Noun::of(b"g"))).unwrap();
        let mut p2 = Layers::new(dir.clone());
        p2.add_layer(layer("gamma", Noun::of(b"g"))).unwrap();
        let mut a = Plan::new(dir.clone(), &mut p1, 0).unwrap();
        let mut b = Plan::new(dir, &mut p2, 0).unwrap();
        a.add_layer(layer("alpha", Noun::of(b"a"))).unwrap();
        b.add_layer(layer("beta", Noun::of(b"b"))).unwrap();
        let ab = a.merge_with(&b).unwrap().seal();
        let ba = b.merge_with(&a).unwrap().seal();
        assert_eq!(ab, ba);
    }
}
