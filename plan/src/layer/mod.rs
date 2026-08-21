use core::clone::Clone;
use core::cmp::{Eq, Ord, PartialEq, PartialOrd};
use core::derive;
use core::fmt::Debug;
use core::hash::Hash;
use core::ops::Drop;
use core::result::Result;
use core::result::Result::Err;
use core::result::Result::Ok;
use heapless::Vec;
use noun::Noun;
pub const MAX_LAYERS: usize = 64;
pub const DEFAULT_INITIAL_PHOENIX_LAYERS: &[Layer] = &[];
/// The maximum length for a layer name.
pub const LAYER_MAX_NAME_LENGTH: usize = 255;

/// The maximum length for a layer description.
pub const LAYER_MAX_DESCRIPTION_LENGTH: usize = 1024;

/// The maximum version number for a layer.
pub const LAYER_MAX_VERSION: u32 = u32::MAX;

/// The minimum version number for a layer.
pub const LAYER_MIN_VERSION: u32 = 0;
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Capabilities {
    Read,
    Write,
    Execute,
    ReadWrite,
    ReadExecute,
    WriteExecute,
    ReadWriteExecute,
    All,
    None,
}
/// The `Layer` struct represents a single layer with a name, version, and description.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Layer {
    pub name: &'static str,
    pub version: u32,
    pub description: &'static str,
    pub root: Noun,
    pub capabilities: Capabilities,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LayerData {
    pub content: &'static [u8],
    pub name: &'static str,
    pub version: u32,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LayerFace {
    Recto(Layers, LibraryLayers),
    Verso(Layers, LibraryLayers),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LibraryLayers {
    pub directory: Noun,
    pub layers: Vec<Layer, MAX_LAYERS>,
}
impl LibraryLayers {
    #[must_use]
    pub const fn new(directory: Noun) -> Self {
        Self {
            directory,
            layers: Vec::new(),
        }
    }

    pub fn add_layer(&mut self, layer: Layer) -> Result<(), &'static str> {
        if self.layers.len() < MAX_LAYERS {
            self.layers.push(layer).map_err(|_| "Failed to add layer")?;
            Ok(())
        } else {
            Err("Maximum number of layers reached")
        }
    }

    pub fn merge_layers(&mut self, other: &Vec<Layer, MAX_LAYERS>) -> Result<(), &'static str> {
        if self.layers.len() + other.len() > MAX_LAYERS {
            return Err("Maximum number of layers exceeded");
        }
        self.layers
            .extend_from_slice(other)
            .map_err(|_| "Failed to merge layers")?;
        Ok(())
    }

    #[must_use]
    pub fn copy_layers(&self) -> Vec<Layer, MAX_LAYERS> {
        self.layers.clone()
    }

    pub fn clear_layers(&mut self) {
        self.layers.clear();
    }

    pub fn set_layers(&mut self, layers: Vec<Layer, MAX_LAYERS>) {
        self.layers = layers;
    }

    #[must_use]
    pub fn get_layers(&self) -> &[Layer] {
        &self.layers
    }
}

impl Drop for Layer {
    fn drop(&mut self) {
        self.name = "";
        self.version = 0;
        self.description = "";
    }
}

/// The `Layers` struct represents a collection of `Layer` instances.
#[repr(C)]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Layers {
    pub phoenix: Vec<Layer, MAX_LAYERS>,
    pub layers: Vec<Layer, MAX_LAYERS>,
    pub should_quit: bool,
    pub directory: Noun,
}
impl Default for Layers {
    fn default() -> Self {
        Self {
            phoenix: Vec::new(),
            layers: Vec::new(),
            should_quit: false,
            directory: Noun::null(),
        }
    }
}

impl Layers {
    #[must_use]
    pub const fn new(directory: Noun) -> Self {
        Self {
            directory,
            phoenix: Vec::new(),
            layers: Vec::new(),
            should_quit: false,
        }
    }

    pub fn add_layer(&mut self, layer: Layer) -> Result<(), &'static str> {
        if self.layers.len() < MAX_LAYERS {
            self.layers.push(layer).map_err(|_| "Failed to add layer")?;
            Ok(())
        } else {
            Err("Maximum number of layers reached")
        }
    }

    // CORRECTION : `other` passé par référence
    pub fn merge_layers(&mut self, other: &Vec<Layer, MAX_LAYERS>) -> Result<(), &'static str> {
        if self.layers.len() + other.len() > MAX_LAYERS {
            return Err("Maximum number of layers exceeded");
        }
        self.layers
            .extend_from_slice(other)
            .map_err(|_| "Failed to merge layers")?;
        Ok(())
    }

    #[must_use] // CORRECTION
    pub fn copy_layers(&self) -> Vec<Layer, MAX_LAYERS> {
        self.layers.clone()
    }

    pub fn clear_layers(&mut self) {
        self.layers.clear();
    }

    pub fn set_layers(&mut self, layers: Vec<Layer, MAX_LAYERS>) {
        self.layers = layers;
    }

    #[must_use] // CORRECTION
    pub fn get_layers(&mut self) -> &[Layer] {
        &self.layers
    }

    /// Merges two `Layer` collections into one `Layers` collection.
    ///
    /// # Arguments
    ///
    /// * `a` - The first `Layer` collection to merge into.
    /// * `b` - The second `Layer` collection to merge into.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the merge was successful.
    /// * `Err(&'static str)` if the maximum number of layers has been exceeded.
    ///
    pub fn merge_layer(&mut self, a: &Layer, b: &Layer) -> Result<(), &'static str> {
        if self.layers.len() + 2 > MAX_LAYERS {
            return Err("Maximum number of layers exceeded");
        }
        let x = heapless::Vec::<Layer, 2>::from_slice(&[a.clone(), b.clone()])
            .map_err(|_| "Failed to create layer slice")?;
        self.layers
            .extend_from_slice(&x)
            .map_err(|_| "Failed to merge layers")?;
        Ok(())
    }
    /// Retrieves a mutable reference to the collection of layers.
    pub const fn get_layers_mut(&mut self) -> &mut Vec<Layer, MAX_LAYERS> {
        &mut self.layers
    }
    /// Adds multiple layers to the `Layers` collection.
    ///
    /// # Arguments
    ///
    /// * `layers` - A `Vec<Layer, MAX_LAYERS>` containing the layers to be added.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the layers were successfully added.
    /// * `Err(&'static str)` if the maximum number of layers has been exceeded.    
    ///
    pub fn add_layers(&mut self, layers: &Vec<Layer, MAX_LAYERS>) -> Result<(), &'static str> {
        if self.layers.len() + layers.len() > MAX_LAYERS {
            return Err("Maximum number of layers exceeded");
        }
        self.layers
            .extend_from_slice(layers)
            .map_err(|_| "Failed to add layers")?;
        Ok(())
    }

    /// Initializes the `Layers` collection with the provided `Vec<Layer, MAX_LAYERS>`.
    ///
    /// # Arguments
    /// * `layers` - A `Vec<Layer, MAX_LAYERS>` containing the initial layers.
    ///
    /// # Returns
    /// * `Ok(())` if the layers were successfully initialized.
    /// * `Err(&'static str)` if the maximum number of layers has been exceeded
    ///
    pub fn init(&mut self, layers: Vec<Layer, MAX_LAYERS>) {
        self.layers.clear();
        self.phoenix.clear();
        self.phoenix = layers;
    }
    /// Retrieves a reference to the phoenix collection of layers.
    pub fn get_phoenix(&mut self) -> &[Layer] {
        &self.phoenix
    }
    /// Reinitializes the `Layers` collection by clearing the current layers and restoring the zeroed layers.
    pub fn reborn(&mut self) {
        self.layers.clear();
        self.layers = self.phoenix.clone();
    }
    /// Retrieves a reference to a specific layer by its index.
    /// # Arguments
    /// * `index` - The index of the layer to retrieve.
    /// # Returns
    /// * A reference to the `Layer` at the specified index.
    #[must_use]
    pub fn get(&self, index: usize) -> &Layer {
        &self.layers[index]
    }
}

impl Layer {
    /// Creates a new `Layer` instance with the given name, version, and description.
    #[must_use]
    pub const fn new(
        name: &'static str,
        version: u32,
        description: &'static str,
        root: Noun,
        capabilities: Capabilities,
    ) -> Self {
        Self {
            name,
            version,
            description,
            root,
            capabilities,
        }
    }
}
