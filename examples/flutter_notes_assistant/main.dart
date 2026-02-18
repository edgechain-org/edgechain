/// EdgeChain – Flutter Notes Assistant Example
///
/// Demonstrates:
///   - Registering a local command (search_notes)
///   - Running an agent with user input
///   - Displaying citations and step trace
///
/// This example uses the Dart-side mock until the flutter_rust_bridge
/// FFI layer is wired up in v0.1.

import 'package:edgechain_flutter/edgechain_flutter.dart';

// ---------------------------------------------------------------------------
// Fake local notes database (replace with sqflite / drift in a real app)
// ---------------------------------------------------------------------------

final _notes = [
  {'id': 'note:1', 'text': 'Met Rajesh. He needs 200 cement bags by next Friday.'},
  {'id': 'note:2', 'text': 'Call supplier about Q1 pricing before Thursday.'},
  {'id': 'note:3', 'text': 'Rajesh also asked about delivery to site B.'},
];

Future<List<Map<String, dynamic>>> _searchNotes(String query) async {
  final q = query.toLowerCase();
  return _notes
      .where((n) => (n['text'] as String).toLowerCase().contains(q))
      .toList();
}

// ---------------------------------------------------------------------------
// EdgeChain setup
// ---------------------------------------------------------------------------

EdgeChainFlutter buildRuntime() {
  final edge = EdgeChainFlutter(
    modelPath: 'assets/models/llama-3.2-3b-instruct-q4_k_m.gguf',
  );

  edge.registerCommand(Command.simple(
    name: 'search_notes',
    description: 'Search notes stored locally on device.',
    category: 'read_only',
    inputSchema: {
      'type': 'object',
      'properties': {
        'query': {'type': 'string'},
        'limit': {'type': 'integer', 'default': 5},
      },
      'required': ['query'],
    },
    handler: (args) async {
      final query = args['query'] as String? ?? '';
      final limit = args['limit'] as int? ?? 5;
      final results = await _searchNotes(query);
      return {'results': results.take(limit).toList()};
    },
  ));

  edge.registerAgent(AgentConfig(
    id: 'notes_assistant',
    systemPrompt: '''
You are a helpful personal assistant with access to the user's local notes.
When asked about past conversations or tasks, use the search_notes command first.
Always cite the note ID when referencing stored information.
Keep answers concise and factual.
''',
    maxSteps: 5,
    allowedCommands: ['search_notes'],
  ));

  return edge;
}

// ---------------------------------------------------------------------------
// Entry point (console demo — swap for Flutter widget tree in a real app)
// ---------------------------------------------------------------------------

Future<void> main() async {
  final edge = buildRuntime();
  final agent = edge.agent('notes_assistant');

  final queries = [
    'What did I discuss with Rajesh last week?',
    'Do I have any supplier calls scheduled?',
    'What is the capital of France?',
  ];

  for (final query in queries) {
    print('\n─────────────────────────────────────');
    print('User: $query');
    final result = await agent.run(query);
    print('Agent: ${result.text}');
    print('Steps: ${result.steps.length}  |  Tokens: ${result.totalTokens}');
    if (result.citations.isNotEmpty) {
      print('Citations:');
      for (final c in result.citations) {
        print('  [${c.sourceId}] ${c.excerpt}');
      }
    }
  }
}
