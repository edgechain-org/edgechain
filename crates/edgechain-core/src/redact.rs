use serde_json::Value;

/// Defines a redaction policy for a specific field or structure.
pub trait RedactionPolicy: Send + Sync {
    /// Redacts sensitive information from the given JSON value.
    fn redact(&self, value: &mut Value);
}

/// A policy that redacts an entire object by replacing it with a placeholder.
pub struct RedactAll(pub String);

impl RedactionPolicy for RedactAll {
    fn redact(&self, value: &mut Value) {
        *value = Value::String(self.0.clone());
    }
}

/// A policy that redacts specific fields from a JSON object.
pub struct RedactFields {
    pub fields: Vec<String>,
    pub placeholder: String,
}

impl RedactFields {
    pub fn new(fields: Vec<impl Into<String>>, placeholder: impl Into<String>) -> Self {
        Self {
            fields: fields.into_iter().map(|f| f.into()).collect(),
            placeholder: placeholder.into(),
        }
    }
}

impl RedactionPolicy for RedactFields {
    fn redact(&self, value: &mut Value) {
        if let Value::Object(map) = value {
            for field in &self.fields {
                if let Some(v) = map.get_mut(field) {
                    *v = Value::String(self.placeholder.clone());
                }
            }
        }
    }
}

/// A registry for applying redaction policies to command outputs before they are processed by the LLM.
#[derive(Default)]
pub struct RedactionRegistry {
    policies: Vec<Box<dyn RedactionPolicy>>,
}

impl RedactionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_policy(&mut self, policy: impl RedactionPolicy + 'static) {
        self.policies.push(Box::new(policy));
    }

    pub fn apply(&self, value: &mut Value) {
        for policy in &self.policies {
            policy.redact(value);
        }
    }
}
