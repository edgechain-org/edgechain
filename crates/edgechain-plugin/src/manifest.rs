use serde::{Deserialize, Serialize};
use edgechain_core::Permission;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub required_permissions: Vec<Permission>,
    pub required_commands: Vec<String>,
    pub provided_commands: Vec<String>,
    pub provided_agents: Vec<String>,
}

impl PluginManifest {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            version: version.into(),
            description: description.into(),
            author: String::new(),
            required_permissions: vec![],
            required_commands: vec![],
            provided_commands: vec![],
            provided_agents: vec![],
        }
    }

    pub fn with_permission(mut self, permission: Permission) -> Self {
        self.required_permissions.push(permission);
        self
    }
}
