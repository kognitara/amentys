#![cfg_attr(not(test), no_std)]

use heapless::Vec;

pub const WA_MAX_NODES: usize = 16;

/// Définit le type d'opération mathématique ou sémantique du nœud
#[derive(Debug, Clone, PartialEq)]
pub enum WaNodeType {
    /// Une brique de base immuable (ex: un calque réseau, une interface)
    BaseLayer,
    /// L'opération de fusion sémantique entre deux autres nœuds de l'AST
    Merge { left_index: usize, right_index: usize },
    /// L'injection d'un paramètre (remplace le concept de WaArgument)
    ContextInject,
}

/// Un composant sémantique (le remplaçant de WaCommand)
#[derive(Debug, Clone)]
pub struct WaNode {
    /// L'empreinte cryptographique (Hash) du composant
    pub hash_id: [u8; 32],
    /// La nature géométrique de ce nœud
    pub node_type: WaNodeType,
}

/// Le Plan global : l'Arbre Syntaxique Abstrait (AST) complet
#[derive(Debug, Clone)]
pub struct WaPlan {
    pub root_hash: [u8; 32],
    /// Le graphe de dépendances et d'opérations (sans allocation dynamique)
    pub nodes: Vec<WaNode, WA_MAX_NODES>,
}

impl WaPlan {
    pub const fn new(root_hash: [u8; 32]) -> Self {
        Self {
            root_hash,
            nodes: Vec::new(),
        }
    }

    /// On n'ajoute plus des "sous-commandes", on ajoute des nœuds structurels
    pub fn add_node(&mut self, node: WaNode) -> Result<(), &'static str> {
        self.nodes
            .push(node)
            .map_err(|_| "Capacité maximale de l'AST (16 nœuds) atteinte")?;
        Ok(())
    }
}