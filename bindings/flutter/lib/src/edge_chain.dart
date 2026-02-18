import 'agent.dart';
import 'command.dart';
import 'memory.dart';
import 'frb_generated.dart' as bridge;

/// Top-level EdgeChain runtime for Flutter.
///
/// Usage:
/// ```dart
/// final edge = EdgeChainFlutter(
///   modelPath: 'assets/models/llama-3.2-3b-q4.gguf',
/// );
/// await edge.init();
/// edge.registerCommand(Command.simple(
///   name: 'search_notes',
///   description: 'Search local notes',
///   handler: (args) async => {'results': await notesDb.search(args['query'])},
/// ));
/// final agent = edge.agent('personal_assistant');
/// final result = await agent.run('What did I discuss with Rajesh last week?');
/// print(result.text);
/// ```
class EdgeChainFlutter {
  final String? modelPath;
  final String? dbPath;

  final Map<String, Command> _commands = {};
  final Map<String, AgentConfig> _agents = {};

  bool _initialized = false;

  EdgeChainFlutter({this.modelPath, this.dbPath});

  /// Initialize the Rust runtime. Must be called before any agent or memory ops.
  Future<void> init() async {
    if (_initialized) return;
    await bridge.edgeInit(dbPath: dbPath);
    _initialized = true;

    // Re-register any commands/agents that were registered before init.
    for (final cmd in _commands.values) {
      _registerCommandBridge(cmd);
    }
    for (final config in _agents.values) {
      _registerAgentBridge(config);
    }
  }

  /// Register a command that agents can call.
  void registerCommand(Command command) {
    _commands[command.meta.name] = command;
    if (_initialized) _registerCommandBridge(command);
  }

  void _registerCommandBridge(Command command) {
    bridge.edgeRegisterCommand(
      name: command.meta.name,
      description: command.meta.description,
      category: command.meta.category,
      inputSchemaJson: command.meta.inputSchema.toString(),
    );
    bridge.edgeSetCommandHandler(
      name: command.meta.name,
      handler: (String argsJson) {
        // Synchronous stub — replaced by real async dispatch after codegen.
        // The actual Dart command handler is invoked via the async bridge.
        return '{"status":"pending"}';
      },
    );
  }

  /// Register a named agent with a specific config.
  void registerAgent(AgentConfig config) {
    _agents[config.id] = config;
    if (_initialized) _registerAgentBridge(config);
  }

  void _registerAgentBridge(AgentConfig config) {
    bridge.edgeRegisterAgent(
      id: config.id,
      systemPrompt: config.systemPrompt,
      maxSteps: config.maxSteps,
      allowedCommands: config.allowedCommands,
    );
  }

  /// Get a named agent, creating one with defaults if not registered.
  Agent agent(String id) {
    final config =
        _agents[id] ??
        AgentConfig(
          id: id,
          systemPrompt: 'You are a helpful assistant named $id.',
        );
    return Agent(config: config, commands: _commands.values.toList());
  }

  /// Memory store backed by the Rust SQLite store via FFI.
  BridgeMemoryStore get memory => BridgeMemoryStore();

  /// All registered command names.
  List<String> get registeredCommands => _commands.keys.toList();
}

/// Memory store that delegates to the Rust bridge.
class BridgeMemoryStore implements MemoryStore {
  @override
  Future<void> set(String key, dynamic value) =>
      bridge.edgeMemorySet(key: key, valueJson: value.toString());

  @override
  Future<dynamic> get(String key) async {
    final entry = await bridge.edgeMemoryGet(key: key);
    return entry?.value;
  }

  @override
  Future<void> delete(String key) => bridge.edgeMemoryDelete(key: key);

  @override
  Future<List<String>> listKeys({String? prefix}) =>
      bridge.edgeMemoryListKeys(prefix: prefix);
}
