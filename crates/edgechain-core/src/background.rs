use async_trait::async_trait;
use crate::{Hook, HookEvent};

/// Trait to define integration points for platform-specific background execution schedulers
/// like Android WorkManager or iOS BGTask.
#[async_trait]
pub trait BackgroundScheduler: Send + Sync {
    /// Schedules a task to be executed in the background.
    /// `task_id` is a unique identifier for the task.
    /// `payload` contains the serialized data required to execute the task.
    async fn schedule(&self, task_id: &str, payload: &str) -> Result<(), String>;
    
    /// Cancels a previously scheduled background task.
    async fn cancel(&self, task_id: &str) -> Result<(), String>;
}

/// A hook that captures events and schedules background tasks when appropriate.
/// For example, if a command fails permanently, it could schedule a background task to retry later when conditions (e.g., network) improve.
pub struct BackgroundSyncHook {
    scheduler: Box<dyn BackgroundScheduler>,
}

impl BackgroundSyncHook {
    pub fn new(scheduler: impl BackgroundScheduler + 'static) -> Self {
        Self {
            scheduler: Box::new(scheduler),
        }
    }
}

#[async_trait]
impl Hook for BackgroundSyncHook {
    fn name(&self) -> &str {
        "background_sync"
    }

    async fn on_event(&self, event: &HookEvent) {
        if let HookEvent::CommandFailedPermanent { name, args, error, .. } = event {
            // Logic to determine if this failure should trigger a background retry.
            // For example, network-related errors could be queued for later.
            let is_network_error = error.to_lowercase().contains("network") || error.to_lowercase().contains("connection");
            
            if is_network_error {
                let task_id = format!("retry_{}_{}", name, uuid::Uuid::new_v4());
                let payload = serde_json::json!({
                    "command": name,
                    "args": args,
                    "original_error": error
                }).to_string();

                if let Err(e) = self.scheduler.schedule(&task_id, &payload).await {
                    tracing::error!(task_id = %task_id, error = %e, "Failed to schedule background task");
                } else {
                    tracing::info!(task_id = %task_id, command = %name, "Scheduled background task for failed command");
                }
            }
        }
    }
}
