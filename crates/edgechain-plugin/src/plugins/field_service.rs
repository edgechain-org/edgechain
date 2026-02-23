use edgechain_core::{AgentConfig, Command, CommandCategory, FnCommandHandler, Skill, SimpleSkill, SkillPolicy};
use crate::{EdgePlugin, EdgeRegistry, PluginManifest};
use serde_json::json;
use std::sync::Arc;

pub struct FieldServicePlugin;

impl EdgePlugin for FieldServicePlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest::new(
            "field_service",
            "Field Service Forms",
            "0.3.0",
            "Helps technicians log offline checklists and reports.",
        )
    }

    fn register(&self, registry: &mut EdgeRegistry) {
        registry.command(Command::new(
            "submit_report",
            "Submit an inspection report or site checklist.",
            CommandCategory::Write,
            json!({ 
                "type": "object", 
                "properties": { 
                    "site_id": { "type": "string" },
                    "status": { "enum": ["pass", "fail", "needs_review"] },
                    "notes": { "type": "string" }
                } 
            }),
            FnCommandHandler(|args: serde_json::Value| async move {
                Ok(json!({ "status": "saved_offline", "site_id": args["site_id"] }))
            }),
        ));

        let field_skill = Arc::new(SimpleSkill::new(
            "site_inspection",
            "Site Inspections & Reports",
            "You are a field service assistant. Help technicians submit accurate offline site reports.",
        ).with_commands(vec!["submit_report"]));

        registry.skill(field_skill.clone());

        registry.agent(
            AgentConfig::new("field_assistant", "You are the field service routing agent.")
        );
    }
}
