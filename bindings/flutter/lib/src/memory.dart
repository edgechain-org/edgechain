/// Dart-side memory API.
/// The actual storage is handled by the Rust core via FFI.
/// This class provides a typed interface for Dart callers.
abstract class MemoryStore {
  Future<void> set(String key, dynamic value);
  Future<dynamic> get(String key);
  Future<void> delete(String key);
  Future<List<String>> listKeys({String? prefix});
}

/// In-process session memory backed by a Dart Map.
/// Cleared when the object is disposed.
class SessionMemory implements MemoryStore {
  final Map<String, dynamic> _store = {};

  @override
  Future<void> set(String key, dynamic value) async {
    _store[key] = value;
  }

  @override
  Future<dynamic> get(String key) async {
    return _store[key];
  }

  @override
  Future<void> delete(String key) async {
    _store.remove(key);
  }

  @override
  Future<List<String>> listKeys({String? prefix}) async {
    if (prefix == null) return _store.keys.toList();
    return _store.keys.where((k) => k.startsWith(prefix)).toList();
  }

  void clear() => _store.clear();
  int get length => _store.length;
}
