#![cfg_attr(not(test), no_std)]

use heapless::Vec;
use noun::Noun;

use crate::layer::{Capabilities, Layer, LayerFace, Layers, LibraryLayers, MAX_LAYERS};

#[doc = "The `layer` module contains the core abstractions and implementations for the plan layer."]
pub mod layer;
#[derive(Debug, Clone)]
pub struct Plan {
    pub branch_index: u8,
    pub directory: Noun,
    pub layers: Vec<Layer, MAX_LAYERS>,
    pub should_quit: bool,
    pub layer_face: LayerFace,
    pub phenix: Layers,
}

impl Plan {
    /// Creates a new instance of `Plan` with an empty list of layers.
    /// Crée un nouveau Plan rattaché à une branche spécifique de awq.
    // CORRECTION : Retourne un Result pour éviter les panic!
    pub fn new(
        directory: Noun,
        phoenix: &mut Layers,
        branch_index: u8,
    ) -> Result<Self, &'static str> {
        if directory.is_null() {
            return Err("Directory Noun cannot be null");
        } else if phoenix.get_layers().is_empty() {
            return Err("Phoenix Layers cannot be empty");
        } else if branch_index > 15 {
            return Err("AWQ Branch Index cannot be greater than 15");
        }

        Ok(Self {
            branch_index,
            layers: Vec::new(),
            should_quit: false,
            layer_face: LayerFace::Recto(phoenix.clone(), LibraryLayers::new(directory.clone())),
            directory,
            phenix: phoenix.clone(),
        })
    }

    #[must_use] // CORRECTION
    pub fn effective_capabilities(&self) -> Capabilities {
        // CORRECTION : map_or au lieu de map().unwrap_or()
        self.layers
            .last()
            .map_or(Capabilities::None, |l| l.capabilities.clone())
    }

    pub fn add_layer(&mut self, layer: Layer) -> Result<(), &'static str> {
        if self.layers.len() < MAX_LAYERS {
            self.layers.push(layer).map_err(|_| "Failed to add layer")?;
            Ok(())
        } else {
            Err("Maximum number of layers reached")
        }
    }

    pub fn remove_layer(&mut self, index: usize) -> Result<(), &'static str> {
        if index < self.layers.len() {
            self.layers.remove(index);
            Ok(())
        } else {
            Err("Index out of bounds")
        }
    }

    pub fn get_layer(&mut self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }

    pub fn set_layers(&mut self, layers: Vec<Layer, MAX_LAYERS>) {
        self.layers = layers;
    }

    #[must_use] // CORRECTION
    pub const fn get_layers(&self) -> &Vec<Layer, MAX_LAYERS> {
        &self.layers
    }

    pub fn clear_layers(&mut self) {
        self.layers.clear();
    }

    #[must_use] // CORRECTION
    pub const fn should_quit(&self) -> bool {
        self.should_quit
    }

    // CORRECTION : const fn
    pub const fn set_should_quit(&mut self, quit: bool) {
        self.should_quit = quit;
    }

    #[must_use] // CORRECTION
    pub fn get_directory(&self) -> Noun {
        self.directory.clone()
    }

    #[must_use] // CORRECTION
    pub fn get_phoenix(&self) -> Layers {
        self.phenix.clone()
    }

    #[must_use] // CORRECTION
    pub const fn directory_is_null(&self) -> bool {
        self.directory.is_null()
    }

    pub fn phoenix_is_empty(&mut self) -> bool {
        self.phenix.get_layers().is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // CORRECTION : Tests modifiés pour vérifier Err au lieu du panic
    #[test]
    fn test_plan_creation_with_null_directory() {
        let mut phoenix: Layers = Layers::default();
        let plan = Plan::new(Noun::new([0u8; 32]), &mut phoenix, 0);
        assert!(plan.is_err());
        assert_eq!(plan.unwrap_err(), "Directory Noun cannot be null");
    }

    #[test]
    fn test_plan_creation_with_empty_phoenix_layers() {
        let mut phoenix: Layers = Layers::default();
        let plan = Plan::new(Noun::new([1u8; 32]), &mut phoenix, 0);
        assert!(plan.is_err());
        assert_eq!(plan.unwrap_err(), "Phoenix Layers cannot be empty");
    }
}
