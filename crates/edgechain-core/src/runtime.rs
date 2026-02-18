use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::info;
use edgechain_model::ModelProvider;
use edgechain_memory::{MemoryStore, SessionMemory};
use crate::{
    agent::{Agent, AgentConfig},
    command::{Command, CommandRegistry},
    error::CoreError,
    hook::{Hook, HookRegistry, TracingHook},
};

/// Top-level EdgeChain runtime. Holds the model, memory, command registry,
/// hook registry, and named agents.
pub struct EdgeChain {
    model: Arc<dyn ModelProvider>,
    memory: Arc<dyn MemoryStore>,
    commands: Arc<CommandRegistry>,
    hooks: Arc<HookRegistry>,
    agents: RwLock<HashMap<String, AgentConfig>>,
}

impl EdgeChain {
    pub fn builder(model: impl ModelProvider + 'static) -> EdgeChainBuilder {
        EdgeChainBuilder::new(model)
    }

    /// Register a command available to all agents.
    /// Can be called at any time, including after plugins are loaded.
    pub fn register_command(&self, command: Command) {
        self.commands.register(command);
    }

    /// Get or create a named agent.
    pub fn agent(&self, id: &str) -> Result<Agent, CoreError> {
        let agents = self.agents.read().unwrap();
        let config = agents.get(id).cloned().unwrap_or_else(|| {
            AgentConfig::new(id, format!("You are a helpful assistant named {id}."))
        });
        Ok(Agent::new(
            config,
            Arc::clone(&self.model),
            Arc::clone(&self.commands),
            Arc::clone(&self.hooks),
            Arc::clone(&self.memory),
        ))
    }

    /// Register a named agent with a specific config.
    pub fn register_agent(&self, config: AgentConfig) {
        info!(id = %config.id, "Registering agent");
        self.agents.write().unwrap().insert(config.id.clone(), config);
    }

    pub fn model(&self) -> &Arc<dyn ModelProvider> {
        &self.model
    }

    pub fn memory(&self) -> &Arc<dyn MemoryStore> {
        &self.memory
    }
}

/// Builder for constructing an `EdgeChain` runtime.
pub struct EdgeChainBuilder {
    model: Arc<dyn ModelProvider>,
    memory: Option<Arc<dyn MemoryStore>>,
    hooks: HookRegistry,
    commands: CommandRegistry,
    enable_tracing_hook: bool,
}

impl EdgeChainBuilder {
    pub fn new(model: impl ModelProvider + 'static) -> Self {
        Self {
            model: Arc::new(model),
            memory: None,
            hooks: HookRegistry::new(),
            commands: CommandRegistry::new(),
            enable_tracing_hook: true,
        }
    }

    pub fn with_memory(mut self, memory: impl MemoryStore + 'static) -> Self {
        self.memory = Some(Arc::new(memory));
        self
    }

    pub fn with_hook(mut self, hook: impl Hook + 'static) -> Self {
        self.hooks.register(hook);
        self
    }

    pub fn with_command(self, command: Command) -> Self {
        self.commands.register(command);
        self
    }

    pub fn without_tracing_hook(mut self) -> Self {
        self.enable_tracing_hook = false;
        self
    }

    pub fn build(mut self) -> EdgeChain {
        if self.enable_tracing_hook {
            self.hooks.register(TracingHook);
        }

        let memory: Arc<dyn MemoryStore> = self.memory
            .unwrap_or_else(|| Arc::new(SessionMemory::new()));

        EdgeChain {
            model: self.model,
            memory,
            commands: Arc::new(self.commands),
            hooks: Arc::new(self.hooks),
            agents: RwLock::new(HashMap::new()),
        }
    }
}
