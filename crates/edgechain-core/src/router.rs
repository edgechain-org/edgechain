use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, debug};

use edgechain_model::{ModelProvider, ModelRequest, ModelMessage, Role};
use crate::error::CoreError;

/// Configuration for a routing rule.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// The intent or topic this route handles (e.g., "inventory", "weather").
    pub intent: String,
    /// The ID of the agent to route to when this intent is detected.
    pub target_agent_id: String,
    /// A description of when this route should be taken.
    pub description: String,
}

/// The result of a routing decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    /// The original user input.
    pub user_input: String,
    /// The selected route intent.
    pub selected_intent: String,
    /// The ID of the agent that should handle this input.
    pub target_agent_id: String,
    /// A confidence score from the model (if applicable/supported), defaults to 1.0.
    pub confidence: f32,
}

/// A router that uses an LLM to classify user input and route it to the appropriate agent.
pub struct AgentRouter {
    model: Arc<dyn ModelProvider>,
    routes: Vec<Route>,
    fallback_agent_id: String,
}

impl AgentRouter {
    pub fn new(model: Arc<dyn ModelProvider>, fallback_agent_id: impl Into<String>) -> Self {
        Self {
            model,
            routes: vec![],
            fallback_agent_id: fallback_agent_id.into(),
        }
    }

    /// Add a new route to the router.
    pub fn with_route(mut self, intent: impl Into<String>, target_agent_id: impl Into<String>, description: impl Into<String>) -> Self {
        self.routes.push(Route {
            intent: intent.into(),
            target_agent_id: target_agent_id.into(),
            description: description.into(),
        });
        self
    }

    /// Evaluate the user input against the configured routes using the LLM.
    pub async fn route(&self, user_input: &str) -> Result<RoutingResult, CoreError> {
        info!(input = %user_input, "Routing user input");

        if self.routes.is_empty() {
            debug!("No routes configured, using fallback agent");
            return Ok(RoutingResult {
                user_input: user_input.to_string(),
                selected_intent: "fallback".to_string(),
                target_agent_id: self.fallback_agent_id.clone(),
                confidence: 1.0,
            });
        }

        let prompt = self.build_routing_prompt(user_input);
        
        let request = ModelRequest::new(vec![
            ModelMessage { role: Role::System, content: prompt },
            ModelMessage { role: Role::User, content: user_input.to_string() }
        ]);

        let response = self.model.complete(request).await.map_err(CoreError::Model)?;
        let content = response.content.trim().to_lowercase();
        
        debug!(response = %content, "Router model response");

        // Parse the response. Expecting just the intent name.
        for route in &self.routes {
            if content.contains(&route.intent.to_lowercase()) {
                info!(intent = %route.intent, target = %route.target_agent_id, "Route selected");
                return Ok(RoutingResult {
                    user_input: user_input.to_string(),
                    selected_intent: route.intent.clone(),
                    target_agent_id: route.target_agent_id.clone(),
                    confidence: 1.0,
                });
            }
        }

        info!(target = %self.fallback_agent_id, "No specific route matched, using fallback");
        Ok(RoutingResult {
            user_input: user_input.to_string(),
            selected_intent: "fallback".to_string(),
            target_agent_id: self.fallback_agent_id.clone(),
            confidence: 1.0,
        })
    }

    fn build_routing_prompt(&self, _user_input: &str) -> String {
        let mut prompt = String::from(
            "You are an intent classifier. Your job is to analyze the user's input and select the single most appropriate category from the list below.\n\nCategories:\n"
        );

        for route in &self.routes {
            prompt.push_str(&format!("- {}: {}\n", route.intent, route.description));
        }

        prompt.push_str("\nRespond ONLY with the exact name of the selected category. Do not include any other text, explanation, or punctuation. If none apply, respond with 'fallback'.");
        prompt
    }
}
