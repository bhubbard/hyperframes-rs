use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const HF_VFX_CHAIN_VERSION: u32 = 1;
pub const HF_VFX_ATTR: &str = "data-vfx-chain";

/// VFX capture mode for a shader effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum HfVfxCapture {
    #[default]
    None,
    SelfCapture,
    Backdrop,
}

/// An instance of a VFX node in a layer's effect chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HfVfxNode {
    pub id: String,
    pub def: String,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

impl HfVfxNode {
    pub fn new(id: impl Into<String>, def: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            def: def.into(),
            params: HashMap::new(),
            enabled: true,
        }
    }

    pub fn with_param(mut self, key: impl Into<String>, val: impl Into<serde_json::Value>) -> Self {
        self.params.insert(key.into(), val.into());
        self
    }
}

/// A serialized chain of VFX nodes applied to a layer (`data-vfx-chain`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HfVfxChain {
    pub version: u32,
    pub nodes: Vec<HfVfxNode>,
}

impl Default for HfVfxChain {
    fn default() -> Self {
        Self {
            version: HF_VFX_CHAIN_VERSION,
            nodes: Vec::new(),
        }
    }
}

impl HfVfxChain {
    pub fn new(nodes: Vec<HfVfxNode>) -> Self {
        Self {
            version: HF_VFX_CHAIN_VERSION,
            nodes,
        }
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_str)
    }
}

/// List of standard supported VFX definitions.
pub const STANDARD_VFX_DEFS: &[&str] = &[
    "fractal-noise",
    "displacement-map",
    "luma-matte",
    "noise",
    "wave-warp",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vfx_chain_roundtrip() {
        let node1 = HfVfxNode::new("n1", "noise")
            .with_param("amount", 0.15)
            .with_param("animated", true);
        let node2 = HfVfxNode::new("n2", "displacement-map").with_param("maxDisplacement", 20.0);

        let chain = HfVfxChain::new(vec![node1, node2]);
        let json = chain.to_json().unwrap();
        let parsed = HfVfxChain::from_json(&json).unwrap();

        assert_eq!(chain, parsed);
        assert_eq!(parsed.nodes.len(), 2);
        assert_eq!(parsed.nodes[0].def, "noise");
    }

    #[test]
    fn test_standard_defs() {
        assert!(STANDARD_VFX_DEFS.contains(&"fractal-noise"));
        assert!(STANDARD_VFX_DEFS.contains(&"wave-warp"));
        assert!(STANDARD_VFX_DEFS.contains(&"luma-matte"));
    }
}
