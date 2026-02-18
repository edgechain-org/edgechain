/// Data models mirroring the Rust structs exposed over FFI.

enum Role { system, user, assistant, tool }

class ModelMessage {
  final Role role;
  final String content;

  const ModelMessage({required this.role, required this.content});

  factory ModelMessage.system(String content) =>
      ModelMessage(role: Role.system, content: content);
  factory ModelMessage.user(String content) =>
      ModelMessage(role: Role.user, content: content);
  factory ModelMessage.assistant(String content) =>
      ModelMessage(role: Role.assistant, content: content);

  Map<String, dynamic> toJson() => {
        'role': role.name,
        'content': content,
      };
}

class AgentResult {
  final String text;
  final List<AgentStep> steps;
  final List<Citation> citations;
  final int totalTokens;

  const AgentResult({
    required this.text,
    required this.steps,
    required this.citations,
    required this.totalTokens,
  });

  factory AgentResult.fromJson(Map<String, dynamic> json) => AgentResult(
        text: json['text'] as String,
        steps: (json['steps'] as List<dynamic>)
            .map((s) => AgentStep.fromJson(s as Map<String, dynamic>))
            .toList(),
        citations: (json['citations'] as List<dynamic>)
            .map((c) => Citation.fromJson(c as Map<String, dynamic>))
            .toList(),
        totalTokens: json['total_tokens'] as int,
      );
}

class AgentStep {
  final int step;
  final String actionType;
  final String observation;

  const AgentStep({
    required this.step,
    required this.actionType,
    required this.observation,
  });

  factory AgentStep.fromJson(Map<String, dynamic> json) => AgentStep(
        step: json['step'] as int,
        actionType: (json['action'] as Map<String, dynamic>)['type'] as String,
        observation: json['observation'] as String,
      );
}

class Citation {
  final String sourceId;
  final String excerpt;

  const Citation({required this.sourceId, required this.excerpt});

  factory Citation.fromJson(Map<String, dynamic> json) => Citation(
        sourceId: json['source_id'] as String,
        excerpt: json['excerpt'] as String,
      );
}

class CommandMeta {
  final String name;
  final String description;
  final String category;
  final Map<String, dynamic> inputSchema;

  const CommandMeta({
    required this.name,
    required this.description,
    required this.category,
    required this.inputSchema,
  });
}
