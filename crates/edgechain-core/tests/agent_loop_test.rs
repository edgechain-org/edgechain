use edgechain_core::{
    AgentConfig, Command, CommandCategory, EdgeChain, FnCommandHandler,
};
use edgechain_memory::MemoryStore;
use edgechain_model::MockModelProvider;
use serde_json::json;

#[tokio::test]
async fn test_agent_direct_response() {
    let model = MockModelProvider::new("mock").with_responses(vec![
        r#"{"action":"respond","text":"The capital of France is Paris."}"#,
    ]);

    let edge = EdgeChain::builder(model).build();
    let agent = edge.agent("test_agent").unwrap();
    let result = agent.run("What is the capital of France?").await.unwrap();

    assert_eq!(result.text, "The capital of France is Paris.");
    assert_eq!(result.steps.len(), 1);
}

#[tokio::test]
async fn test_agent_command_call_then_respond() {
    let model = MockModelProvider::new("mock").with_responses(vec![
        r#"{"action":"call_command","name":"get_weather","args":{"city":"London"}}"#,
        r#"{"action":"respond","text":"The weather in London is sunny."}"#,
    ]);

    let edge = EdgeChain::builder(model)
        .with_command(Command::new(
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
                Ok(json!({ "city": city, "condition": "sunny", "temp_c": 18 }))
            }),
        ))
        .build();

    let agent = edge.agent("weather_agent").unwrap();
    let result = agent.run("What is the weather in London?").await.unwrap();

    assert_eq!(result.text, "The weather in London is sunny.");
    assert!(result.steps.len() >= 2);
}

#[tokio::test]
async fn test_agent_step_limit_error() {
    let model = MockModelProvider::new("mock").with_responses(vec![
        r#"{"action":"call_command","name":"loop_cmd","args":{}}"#,
    ]);

    let edge = EdgeChain::builder(model)
        .with_command(Command::new(
            "loop_cmd",
            "A command that always triggers another step.",
            CommandCategory::ReadOnly,
            json!({}),
            FnCommandHandler(|_| async move { Ok(json!({ "status": "ok" })) }),
        ))
        .build();

    let mut config = AgentConfig::new("looping_agent", "You loop forever.");
    config.max_steps = 3;
    edge.register_agent(config);

    let agent = edge.agent("looping_agent").unwrap();
    let result = agent.run("Loop forever").await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("step limit"), "Expected step limit error, got: {err}");
}

#[tokio::test]
async fn test_agent_command_allowlist_blocks_unknown_command() {
    let model = MockModelProvider::new("mock").with_responses(vec![
        r#"{"action":"call_command","name":"dangerous_cmd","args":{}}"#,
        r#"{"action":"respond","text":"I cannot do that."}"#,
    ]);

    let edge = EdgeChain::builder(model).build();

    let config = AgentConfig::new("safe_agent", "You are a safe agent.")
        .with_allowed_commands(vec!["safe_cmd"]);
    edge.register_agent(config);

    let agent = edge.agent("safe_agent").unwrap();
    let result = agent.run("Do something dangerous").await.unwrap();

    assert_eq!(result.text, "I cannot do that.");
}

#[tokio::test]
async fn test_session_memory_persists_within_run() {
    use edgechain_memory::SessionMemory;
    use std::sync::Arc;

    let memory = Arc::new(SessionMemory::new());
    memory
        .set("user_name", serde_json::json!("Praveen"))
        .await
        .unwrap();

    let stored = memory.get("user_name").await.unwrap();
    assert!(stored.is_some());
    assert_eq!(stored.unwrap().value, serde_json::json!("Praveen"));
}

#[tokio::test]
async fn test_sqlite_memory_roundtrip() {
    use edgechain_memory::{MemoryStore, SqliteMemoryStore};

    let store = SqliteMemoryStore::in_memory().unwrap();
    store
        .set("key1", json!({ "name": "EdgeChain", "version": 1 }))
        .await
        .unwrap();

    let entry = store.get("key1").await.unwrap().unwrap();
    assert_eq!(entry.key, "key1");
    assert_eq!(entry.value["name"], "EdgeChain");

    let keys = store.list_keys(None).await.unwrap();
    assert!(keys.contains(&"key1".to_string()));

    store.delete("key1").await.unwrap();
    assert!(store.get("key1").await.unwrap().is_none());
}
