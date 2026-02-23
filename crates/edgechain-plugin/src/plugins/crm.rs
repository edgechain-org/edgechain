use edgechain_core::{AgentConfig, Command, CommandCategory, FnCommandHandler, Skill, SimpleSkill, SkillPolicy};
use crate::{EdgePlugin, EdgeRegistry, PluginManifest};
use serde_json::json;
use std::sync::Arc;

pub struct CrmPlugin;

impl EdgePlugin for CrmPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest::new(
            "crm",
            "CRM Assistant",
            "0.3.0",
            "Manages customer contacts, meetings, and follow-ups.",
        )
    }

    fn register(&self, registry: &mut EdgeRegistry) {
        registry.command(Command::new(
            "get_contact",
            "Lookup customer contact details by name.",
            CommandCategory::ReadOnly,
            json!({ "type": "object", "properties": { "name": { "type": "string" } } }),
            FnCommandHandler(|args: serde_json::Value| async move {
                let name = args["name"].as_str().unwrap_or("Unknown");
                Ok(json!({ "name": name, "email": format!("{}@example.com", name.to_lowercase()), "phone": "555-0100" }))
            }),
        ));

        registry.command(Command::new(
            "log_meeting",
            "Record notes from a customer meeting.",
            CommandCategory::Write,
            json!({ 
                "type": "object", 
                "properties": { 
                    "customer_name": { "type": "string" },
                    "notes": { "type": "string" }
                } 
            }),
            FnCommandHandler(|args: serde_json::Value| async move {
                Ok(json!({ "status": "success", "logged_for": args["customer_name"] }))
            }),
        ));

        let crm_skill = Arc::new(SimpleSkill::new(
            "crm_basics",
            "Customer Relationship Management",
            "You are a CRM assistant. You help users look up contact info and log meeting notes. Always be professional.",
        ).with_commands(vec!["get_contact", "log_meeting"]));

        registry.skill(crm_skill.clone());

        registry.agent(
            AgentConfig::new("crm_assistant", "You are the CRM routing agent.")
        );
    }
}
