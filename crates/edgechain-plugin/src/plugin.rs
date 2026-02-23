use edgechain_core::{Command, AgentConfig, Hook};
use crate::manifest::PluginManifest;
use std::sync::Arc;
use edgechain_core::Skill;

/// The registry passed to a plugin during registration.
/// Collects everything the plugin contributes.
#[derive(Default)]
pub struct EdgeRegistry {
    pub commands: Vec<Command>,
    pub agents: Vec<AgentConfig>,
    pub hooks: Vec<Box<dyn Hook>>,
    pub skills: Vec<Arc<dyn Skill>>,
}

impl EdgeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn command(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn agent(&mut self, config: AgentConfig) {
        self.agents.push(config);
    }

    pub fn hook(&mut self, hook: impl Hook + 'static) {
        self.hooks.push(Box::new(hook));
    }

    pub fn skill(&mut self, skill: Arc<dyn Skill>) {
        self.skills.push(skill);
    }
}

/// Trait every EdgeChain plugin must implement.
pub trait EdgePlugin: Send + Sync {
    fn manifest(&self) -> PluginManifest;
    fn register(&self, registry: &mut EdgeRegistry);
}
