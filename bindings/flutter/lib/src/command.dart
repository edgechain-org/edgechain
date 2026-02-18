import 'models.dart';

/// Dart-side representation of a command that can be called by an agent.
/// The [handler] is a Dart function invoked when the model requests this command.
class Command {
  final CommandMeta meta;
  final Future<Map<String, dynamic>> Function(Map<String, dynamic> args) handler;

  const Command({required this.meta, required this.handler});

  factory Command.simple({
    required String name,
    required String description,
    String category = 'read_only',
    Map<String, dynamic> inputSchema = const {},
    required Future<Map<String, dynamic>> Function(Map<String, dynamic>) handler,
  }) {
    return Command(
      meta: CommandMeta(
        name: name,
        description: description,
        category: category,
        inputSchema: inputSchema,
      ),
      handler: handler,
    );
  }
}
