use edgechain_core::{AgentConfig, Command, CommandCategory, EdgeChain, FnCommandHandler};
use edgechain_model::MockModelProvider;
use edgechain_plugin::{EdgePlugin, EdgeRegistry, PluginManifest, PluginManager};
use serde_json::json;

struct WeatherPlugin;

impl EdgePlugin for WeatherPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest::new(
            "weather",
            "Weather Plugin",
            "0.1.0",
            "Provides weather commands and a weather agent.",
        )
    }

    fn register(&self, registry: &mut EdgeRegistry) {
        registry.command(Command::new(
            "get_weather",
            "Get current weather for a city.",
            CommandCategory::ReadOnly,
            json!({
                "type": "object",
                "properties": { "city": { "type": "string" } },
                "required": ["city"]
            }),
            FnCommandHandler(|args: serde_json::Value| async move {
                let city = args["city"].as_str().unwrap_or("unknown");
                Ok(json!({ "city": city, "condition": "sunny", "temp_c": 22 }))
            }),
        ));

        registry.agent(
            AgentConfig::new("weather_agent", "You are a weather assistant.")
                .with_allowed_commands(vec!["get_weather"]),
        );
    }
}

#[tokio::test]
async fn test_plugin_registers_command_and_agent() {
    let model = MockModelProvider::new("mock").with_responses(vec![
        r#"{"action":"call_command","name":"get_weather","args":{"city":"Tokyo"}}"#,
        r#"{"action":"respond","text":"It is sunny in Tokyo at 22°C."}"#,
    ]);

    let edge = EdgeChain::builder(model).build();
    let mut manager = PluginManager::new();

    manager.load(&WeatherPlugin, &edge).unwrap();

    assert!(manager.is_loaded("weather"));
    assert_eq!(manager.loaded_plugins().len(), 1);

    let agent = edge.agent("weather_agent").unwrap();
    let result = agent.run("What is the weather in Tokyo?").await.unwrap();

    assert_eq!(result.text, "It is sunny in Tokyo at 22°C.");
}

#[tokio::test]
async fn test_plugin_cannot_be_loaded_twice() {
    let model = MockModelProvider::new("mock");
    let edge = EdgeChain::builder(model).build();
    let mut manager = PluginManager::new();

    manager.load(&WeatherPlugin, &edge).unwrap();
    let result = manager.load(&WeatherPlugin, &edge);

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("already registered"), "Expected already-registered error, got: {err}");
}
