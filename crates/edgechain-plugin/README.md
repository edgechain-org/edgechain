# edgechain-plugin

Plugin trait and manager for the [EdgeChain](https://github.com/edgechain-org/edgechain) SDK.

## What's in this crate

- **`EdgePlugin`** trait — implement to bundle commands + agents + hooks into a distributable unit
- **`PluginManifest`** — name, version, description metadata
- **`PluginManager`** — load/unload plugins, duplicate detection
- **`EdgeRegistry`** — collects what a plugin contributes (commands, agents, hooks)

## Quick start

```toml
[dependencies]
edgechain-plugin = "0.1"
edgechain-core = "0.1"
edgechain-model = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
serde_json = "1"
```

```rust
use edgechain_core::{AgentConfig, Command, CommandCategory, EdgeChain, FnCommandHandler};
use edgechain_model::MockModelProvider;
use edgechain_plugin::{EdgePlugin, EdgeRegistry, PluginManifest, PluginManager};
use serde_json::json;

struct WeatherPlugin;

impl EdgePlugin for WeatherPlugin {
    fn manifest(&self) -> PluginManifest {
        PluginManifest::new("weather", "Weather Plugin", "0.1.0", "Provides weather data.")
    }

    fn register(&self, registry: &mut EdgeRegistry) {
        registry.command(Command::new(
            "get_weather",
            "Get current weather for a city.",
            CommandCategory::ReadOnly,
            json!({ "type": "object", "properties": { "city": { "type": "string" } } }),
            FnCommandHandler(|args: serde_json::Value| async move {
                Ok(json!({ "city": args["city"], "condition": "sunny" }))
            }),
        ));
        registry.agent(
            AgentConfig::new("weather_agent", "You are a weather assistant.")
                .with_allowed_commands(vec!["get_weather"]),
        );
    }
}

#[tokio::main]
async fn main() {
    let edge = EdgeChain::builder(MockModelProvider::new("mock")).build();
    let mut manager = PluginManager::new();
    manager.load(&WeatherPlugin, &edge).unwrap();

    let agent = edge.agent("weather_agent").unwrap();
    let result = agent.run("What is the weather in Tokyo?").await.unwrap();
    println!("{}", result.text);
}
```

## License

MIT OR Apache-2.0
