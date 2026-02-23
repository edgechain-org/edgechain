use edgechain_core::{AgentConfig, Command, CommandCategory, FnCommandHandler, Skill, SimpleSkill, SkillPolicy};
use crate::{EdgePlugin, EdgeRegistry, PluginManifest};
use serde_json::json;
use std::sync::Arc;

pub struct InventoryPlugin;

impl EdgePlugin for InventoryPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest::new(
            "inventory",
            "Inventory Tracker",
            "0.3.0",
            "Look up stock levels and trigger purchase orders.",
        )
    }

    fn register(&self, registry: &mut EdgeRegistry) {
        registry.command(Command::new(
            "check_stock",
            "Look up the current stock level for a SKU or product name.",
            CommandCategory::ReadOnly,
            json!({ "type": "object", "properties": { "item": { "type": "string" } } }),
            FnCommandHandler(|args: serde_json::Value| async move {
                let item = args["item"].as_str().unwrap_or("unknown");
                Ok(json!({ "item": item, "in_stock": 42 }))
            }),
        ));

        registry.command(Command::new(
            "order_stock",
            "Create a purchase order for an item.",
            CommandCategory::Write,
            json!({ 
                "type": "object", 
                "properties": { 
                    "item": { "type": "string" },
                    "quantity": { "type": "integer" }
                } 
            }),
            FnCommandHandler(|args: serde_json::Value| async move {
                Ok(json!({ "status": "order_placed", "qty": args["quantity"] }))
            }),
        ));

        let inventory_skill = Arc::new(SimpleSkill::new(
            "inventory_ops",
            "Inventory Management",
            "You are an inventory assistant. You can check stock and place orders. When placing orders, always confirm the quantity.",
        ).with_commands(vec!["check_stock", "order_stock"])
         .with_policy(SkillPolicy::RequiresConfirmation));

        registry.skill(inventory_skill.clone());

        registry.agent(
            AgentConfig::new("inventory_assistant", "You are the inventory agent.")
        );
    }
}
