use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};
use edgechain_model::{ModelProvider, ModelRequest};
use edgechain_memory::MemoryStore;
use edgechain_rag::Retriever;
use crate::{
    command::CommandRegistry,
    context::{AgentContext, ParsedAction, parse_model_output},
    error::CoreError,
    hook::{HookEvent, HookRegistry, ModelCallContext, ModelOutputContext, ErrorContext},
    skill::SkillSet,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub system_prompt: String,
    pub max_steps: usize,
    pub allowed_commands: Option<Vec<String>>,
}

impl AgentConfig {
    pub fn new(id: impl Into<String>, system_prompt: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            system_prompt: system_prompt.into(),
            max_steps: 10,
            allowed_commands: None,
        }
    }

    pub fn with_max_steps(mut self, steps: usize) -> Self {
        self.max_steps = steps;
        self
    }

    pub fn with_allowed_commands(mut self, commands: Vec<impl Into<String>>) -> Self {
        self.allowed_commands = Some(commands.into_iter().map(|c| c.into()).collect());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub step: usize,
    pub action: StepAction,
    pub observation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StepAction {
    ModelCall { prompt_tokens: usize },
    CommandCall { name: String, args: serde_json::Value },
    Retrieve { query: String },
    FinalResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub text: String,
    pub steps: Vec<AgentStep>,
    pub citations: Vec<Citation>,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub source_id: String,
    pub excerpt: String,
}

pub struct Agent {
    pub config: AgentConfig,
    model: Arc<dyn ModelProvider>,
    commands: Arc<CommandRegistry>,
    hooks: Arc<HookRegistry>,
    memory: Arc<dyn MemoryStore>,
    retriever: Option<Arc<dyn Retriever>>,
    skills: SkillSet,
}

impl Agent {
    pub fn new(
        config: AgentConfig,
        model: Arc<dyn ModelProvider>,
        commands: Arc<CommandRegistry>,
        hooks: Arc<HookRegistry>,
        memory: Arc<dyn MemoryStore>,
    ) -> Self {
        Self { config, model, commands, hooks, memory, retriever: None, skills: SkillSet::new() }
    }

    pub fn with_retriever(mut self, retriever: Arc<dyn Retriever>) -> Self {
        self.retriever = Some(retriever);
        self
    }

    pub fn with_skill(mut self, skill: Arc<dyn crate::skill::Skill>) -> Self {
        self.skills.add(skill);
        self
    }

    pub async fn run(&self, user_input: &str) -> Result<AgentResult, CoreError> {
        let session_id = uuid::Uuid::new_v4().to_string();
        info!(agent = %self.config.id, session = %session_id, input = %user_input, "Agent run started");

        let system_prompt = if self.skills.is_empty() {
            self.config.system_prompt.clone()
        } else {
            format!("{}{}", self.config.system_prompt, self.skills.build_system_prompt())
        };

        let mut ctx = AgentContext::new(
            &self.config.id,
            &session_id,
            &system_prompt,
            Arc::clone(&self.memory),
        );
        ctx.push_user(user_input);

        let mut steps: Vec<AgentStep> = vec![];
        let mut total_tokens = 0usize;
        let mut citations: Vec<Citation> = vec![];

        for step_num in 0..self.config.max_steps {
            debug!(agent = %self.config.id, step = step_num, "Agent step");

            let request = ModelRequest::new(ctx.conversation.clone());
            let prompt_tokens = ctx.conversation.iter().map(|m| m.content.split_whitespace().count()).sum();

            self.hooks.emit(HookEvent::BeforeModelCall(ModelCallContext {
                agent_id: self.config.id.clone(),
                step: step_num,
                prompt_tokens,
            })).await;

            let response = self.model.complete(request).await.map_err(|e| {
                CoreError::Model(e)
            })?;

            total_tokens += response.tokens_used;

            self.hooks.emit(HookEvent::AfterModelCall(ModelOutputContext {
                agent_id: self.config.id.clone(),
                step: step_num,
                tokens_used: response.tokens_used,
                content: response.content.clone(),
            })).await;

            steps.push(AgentStep {
                step: step_num,
                action: StepAction::ModelCall { prompt_tokens },
                observation: response.content.clone(),
            });

            let action = parse_model_output(&response.content);

            match action {
                ParsedAction::Respond { text } => {
                    ctx.push_assistant(&text);
                    info!(agent = %self.config.id, steps = step_num + 1, tokens = total_tokens, "Agent run complete");
                    return Ok(AgentResult {
                        text,
                        steps,
                        citations,
                        total_tokens,
                    });
                }

                ParsedAction::CallCommand { name, args } => {
                    let mut allowed = self.config.allowed_commands.clone().unwrap_or_default();
                    allowed.extend(self.skills.allowed_commands());

                    if !allowed.is_empty() && !allowed.contains(&name) {
                        warn!(agent = %self.config.id, command = %name, "Command not in allowlist");
                        ctx.push_tool_result(format!("Error: command '{name}' is not allowed for this agent."));
                        continue;
                    }

                    self.hooks.emit(HookEvent::BeforeCommand { name: name.clone(), args: args.clone() }).await;
                    let result = self.commands.execute(&name, args.clone()).await?;
                    self.hooks.emit(HookEvent::AfterCommand(result.clone())).await;

                    let observation = serde_json::to_string_pretty(&result.output)?;
                    ctx.push_tool_result(format!("Command '{name}' result:\n{observation}"));

                    steps.push(AgentStep {
                        step: step_num,
                        action: StepAction::CommandCall { name, args },
                        observation: observation.clone(),
                    });
                }

                ParsedAction::Retrieve { query } => {
                    let observation = if let Some(retriever) = &self.retriever {
                        let chunks = retriever.search(&query, 5).await
                            .unwrap_or_default();

                        if chunks.is_empty() {
                            format!("No results found for '{query}'.")
                        } else {
                            let mut context_text = format!("Retrieved {n} chunks for '{query}':\n\n", n = chunks.len());
                            for (i, chunk) in chunks.iter().enumerate() {
                                context_text.push_str(&format!("[{i}] (source: {id})\n{text}\n\n",
                                    id = chunk.id,
                                    text = chunk.text,
                                ));
                                citations.push(Citation {
                                    source_id: chunk.id.clone(),
                                    excerpt: chunk.text.chars().take(200).collect(),
                                });
                            }
                            context_text
                        }
                    } else {
                        format!("No retriever configured. Answer from your existing knowledge for '{query}'.")
                    };

                    ctx.push_tool_result(observation.clone());
                    steps.push(AgentStep {
                        step: step_num,
                        action: StepAction::Retrieve { query: query.clone() },
                        observation,
                    });
                }
            }
        }

        self.hooks.emit(HookEvent::Error(ErrorContext {
            agent_id: self.config.id.clone(),
            step: self.config.max_steps,
            error: "Step limit reached".to_string(),
        })).await;

        Err(CoreError::StepLimitReached(self.config.max_steps))
    }
}
