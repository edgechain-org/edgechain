use std::sync::Arc;

/// Policies that dictate how a skill can be used by an agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillPolicy {
    /// No restrictions.
    Unrestricted,
    /// The skill only performs read operations. Agents can use it freely without mutating state.
    ReadOnly,
    /// Commands within this skill require explicit user confirmation before execution.
    RequiresConfirmation,
}

/// A Skill bundles a system prompt, a set of allowed commands, and policies.
/// Skills are given to agents to dynamically expand their capabilities.
pub trait Skill: Send + Sync {
    /// The unique name of the skill (e.g., "inventory_management").
    fn name(&self) -> &str;
    
    /// A human-readable description of what this skill enables.
    fn description(&self) -> &str;
    
    /// The system prompt instructions injected into the agent's context when this skill is active.
    fn system_prompt(&self) -> String;
    
    /// The names of the commands this skill allows the agent to use.
    fn allowed_commands(&self) -> Vec<String>;
    
    /// The policy enforcing how this skill's commands are executed.
    fn policy(&self) -> SkillPolicy {
        SkillPolicy::Unrestricted
    }
}

/// A basic implementation of a [`Skill`] using static data.
pub struct SimpleSkill {
    name: String,
    description: String,
    system_prompt: String,
    allowed_commands: Vec<String>,
    policy: SkillPolicy,
}

impl SimpleSkill {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        system_prompt: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            system_prompt: system_prompt.into(),
            allowed_commands: vec![],
            policy: SkillPolicy::Unrestricted,
        }
    }

    pub fn with_commands(mut self, commands: Vec<impl Into<String>>) -> Self {
        self.allowed_commands = commands.into_iter().map(|c| c.into()).collect();
        self
    }

    pub fn with_policy(mut self, policy: SkillPolicy) -> Self {
        self.policy = policy;
        self
    }
}

impl Skill for SimpleSkill {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn system_prompt(&self) -> String {
        self.system_prompt.clone()
    }

    fn allowed_commands(&self) -> Vec<String> {
        self.allowed_commands.clone()
    }

    fn policy(&self) -> SkillPolicy {
        self.policy.clone()
    }
}

/// A registry/collection of skills assigned to an agent.
pub struct SkillSet {
    skills: Vec<Arc<dyn Skill>>,
}

impl SkillSet {
    pub fn new() -> Self {
        Self { skills: vec![] }
    }

    pub fn add(&mut self, skill: Arc<dyn Skill>) {
        self.skills.push(skill);
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    /// Combines all skill system prompts into a single instructional block.
    pub fn build_system_prompt(&self) -> String {
        if self.skills.is_empty() {
            return String::new();
        }

        let mut prompt = String::from("\n\n# Active Skills\nYou have access to the following skills:\n");
        for skill in &self.skills {
            prompt.push_str(&format!("\n## Skill: {}\n{}\n", skill.name(), skill.system_prompt()));
        }
        prompt
    }

    /// Returns a combined list of all commands allowed by the active skills.
    pub fn allowed_commands(&self) -> Vec<String> {
        let mut commands = vec![];
        for skill in &self.skills {
            commands.extend(skill.allowed_commands());
        }
        commands.sort();
        commands.dedup();
        commands
    }
}

impl Default for SkillSet {
    fn default() -> Self {
        Self::new()
    }
}
