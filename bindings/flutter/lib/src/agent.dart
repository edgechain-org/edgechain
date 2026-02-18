import 'models.dart';
import 'command.dart';

/// Configuration for a named agent.
class AgentConfig {
  final String id;
  final String systemPrompt;
  final int maxSteps;
  final List<String>? allowedCommands;

  const AgentConfig({
    required this.id,
    required this.systemPrompt,
    this.maxSteps = 10,
    this.allowedCommands,
  });
}

/// A running agent instance. Wraps the Rust agent via FFI.
/// Until FFI bridge is generated, [run] executes a mock loop in Dart.
class Agent {
  final AgentConfig config;
  // ignore: unused_field
  final List<Command> _commands;

  Agent({required this.config, List<Command> commands = const []})
    : _commands = commands;

  /// Run the agent with [userInput] and return the result.
  Future<AgentResult> run(String userInput) async {
    // TODO: replace with flutter_rust_bridge FFI call once bridge is generated.
    // For now, performs a single mock step.
    final steps = <AgentStep>[];

    steps.add(
      AgentStep(
        step: 0,
        actionType: 'model_call',
        observation: '[MockAgent] Received: $userInput',
      ),
    );

    return AgentResult(
      text:
          '[MockAgent] I received your message: "$userInput". '
          'Real inference will be available once the llama.cpp provider is wired up.',
      steps: steps,
      citations: [],
      totalTokens: userInput.split(' ').length,
    );
  }
}
