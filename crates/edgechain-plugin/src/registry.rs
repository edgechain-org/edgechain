use std::collections::HashMap;
use tracing::info;
use edgechain_core::EdgeChain;
use crate::{
    error::PluginError,
    manifest::PluginManifest,
    plugin::{EdgePlugin, EdgeRegistry},
};

/// Manages loading and unloading of plugins into an EdgeChain runtime.
pub struct PluginManager {
    loaded: HashMap<String, PluginManifest>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self { loaded: HashMap::new() }
    }

    pub fn load(
        &mut self,
        plugin: &dyn EdgePlugin,
        runtime: &EdgeChain,
    ) -> Result<(), PluginError> {
        let manifest = plugin.manifest();

        if self.loaded.contains_key(&manifest.id) {
            return Err(PluginError::AlreadyRegistered(manifest.id.clone()));
        }

        info!(plugin = %manifest.id, version = %manifest.version, "Loading plugin");

        let mut reg = EdgeRegistry::new();
        plugin.register(&mut reg);

        for command in reg.commands {
            runtime.register_command(command);
        }
        for agent_config in reg.agents {
            runtime.register_agent(agent_config);
        }

        self.loaded.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    pub fn is_loaded(&self, id: &str) -> bool {
        self.loaded.contains_key(id)
    }

    pub fn loaded_plugins(&self) -> Vec<&PluginManifest> {
        self.loaded.values().collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}
