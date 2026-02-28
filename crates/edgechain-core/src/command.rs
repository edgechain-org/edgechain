use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{debug, warn};
use crate::{error::CoreError, retry::{RetryPolicy, NoRetry}};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CommandCategory {
    ReadOnly,
    Write,
    External,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMeta {
    pub name: String,
    pub description: String,
    pub category: CommandCategory,
    pub input_schema: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandResult {
    pub command: String,
    pub output: Value,
    pub success: bool,
    pub error: Option<String>,
}

impl CommandResult {
    pub fn ok(command: impl Into<String>, output: Value) -> Self {
        Self { command: command.into(), output, success: true, error: None }
    }

    pub fn err(command: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            output: Value::Null,
            success: false,
            error: Some(error.into()),
        }
    }
}

#[async_trait]
pub trait CommandHandler: Send + Sync {
    async fn execute(&self, args: Value) -> Result<Value, CoreError>;
}

pub struct Command {
    pub meta: CommandMeta,
    handler: Arc<dyn CommandHandler>,
    retry_policy: Arc<dyn RetryPolicy>,
}

impl Command {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        category: CommandCategory,
        input_schema: Value,
        handler: impl CommandHandler + 'static,
    ) -> Self {
        Self {
            meta: CommandMeta {
                name: name.into(),
                description: description.into(),
                category,
                input_schema,
            },
            handler: Arc::new(handler),
            retry_policy: Arc::new(NoRetry),
        }
    }

    pub fn with_retry_policy(mut self, policy: impl RetryPolicy + 'static) -> Self {
        self.retry_policy = Arc::new(policy);
        self
    }

    pub async fn run(&self, args: Value) -> CommandResult {
        debug!(command = %self.meta.name, "Executing command");
        match self.handler.execute(args).await {
            Ok(output) => CommandResult::ok(&self.meta.name, output),
            Err(e) => {
                warn!(command = %self.meta.name, error = %e, "Command failed");
                CommandResult::err(&self.meta.name, e.to_string())
            }
        }
    }
}

/// Central registry for all commands available to agents.
/// Uses `RwLock` internally so commands can be registered after the registry
/// is wrapped in an `Arc` (e.g. by plugins loaded at runtime).
pub struct CommandRegistry {
    commands: RwLock<HashMap<String, Command>>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self { commands: RwLock::new(HashMap::new()) }
    }
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, command: Command) {
        debug!(name = %command.meta.name, "Registering command");
        self.commands.write().unwrap().insert(command.meta.name.clone(), command);
    }

    pub fn list(&self) -> Vec<CommandMeta> {
        self.commands.read().unwrap().values().map(|c| c.meta.clone()).collect()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.commands.read().unwrap().contains_key(name)
    }

    pub async fn execute(&self, name: &str, args: Value) -> Result<CommandResult, CoreError> {
        let (handler, retry_policy) = {
            let guard = self.commands.read().unwrap();
            let cmd = guard.get(name).ok_or_else(|| CoreError::CommandNotFound(name.to_string()))?;
            (Arc::clone(&cmd.handler), Arc::clone(&cmd.retry_policy))
        };
        let cmd_name = name.to_string();
        debug!(command = %cmd_name, "Executing command");
        
        let mut attempt = 0;
        loop {
            match handler.execute(args.clone()).await {
                Ok(output) => return Ok(CommandResult::ok(cmd_name, output)),
                Err(e) => {
                    if let Some(delay) = retry_policy.should_retry(attempt, &e) {
                        warn!(command = %cmd_name, attempt = attempt + 1, error = %e, "Command failed, retrying after delay");
                        tokio::time::sleep(delay).await;
                        attempt += 1;
                    } else {
                        warn!(command = %cmd_name, attempts = attempt + 1, error = %e, "Command failed permanently");
                        return Ok(CommandResult::err(cmd_name, e.to_string()));
                    }
                }
            }
        }
    }
}

/// Convenience: build a command from a plain async closure.
pub struct FnCommandHandler<F>(pub F);

#[async_trait]
impl<F, Fut> CommandHandler for FnCommandHandler<F>
where
    F: Fn(Value) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Value, CoreError>> + Send,
{
    async fn execute(&self, args: Value) -> Result<Value, CoreError> {
        (self.0)(args).await
    }
}
