import 'package:app/slotmap.dart';
import 'package:vector_math/vector_math.dart';

class Transform {
  Vector3 position = Vector3.zero();
  Quaternion rotation = Quaternion.identity();
  Vector3 scale = Vector3.all(1.0);

  @override
  String toString() {
    return 'Transform(position: ${position.toString()}, rotation: ${rotation.toString()}, scale: ${scale.toString()})';
  }
}

class Entity {
  Transform transform = Transform();
  Transform globalTransform = Transform();

  @override
  String toString() {
    return 'Entity(transform: ${transform.toString()}, globalTransform: ${globalTransform.toString()})';
  }
}

class Parent {
  Key? key;
  Parent(this.key);

  @override
  bool operator ==(Object other) {
    if (other is Parent) {
      return key == other.key;
    }
    return false;
  }

  @override
  int get hashCode => key.hashCode;
}

class World {
  final _entities = SlotMap<Entity>();
  final _children = SlotMap<List<Key>>();
  final _parents = SlotMap<Parent>();
  final _resources = <Type, dynamic>{};

  void insertResource<T>(T resource) {
    _resources[resource.runtimeType] = resource;
  }

  void removeResource<T>() {
    _resources.remove(T);
  }

  T getResource<T>() {
    return _resources[T];
  }

  Key spawn() {
    final entity = _entities.insert(Entity());
    // Must be growable since we mutate it (add/remove children).
    final _ = _children.insert(<Key>[]);
    final _ = _parents.insert(Parent(null));
    return entity;
  }

  void addChild(Key parent, Key child) {
    final parentChildren = _children.get(parent)!;
    parentChildren.add(child);
    _parents.set(child, Parent(parent));
  }

  void remove(Key entity) {
    final parent = _parents.get(entity)!.key;
    if (parent != null) {
      final childList = _children.get(parent)!;
      childList.remove(entity);
    }

    // Remove "parent" references to a node when removing that node
    final childList = _children.get(entity)!;
    for (final child in childList) {
      _parents.set(child, Parent(null));
    }

    final _ = _children.remove(entity);
    final _ = _parents.remove(entity);
    final _ = _entities.remove(entity);
  }

  Entity? get(Key entity) {
    return _entities.get(entity);
  }

  @override
  String toString() {
    final sb = StringBuffer();
    sb.writeln('World(entities: ${_entities.length()})');

    final allKeys = _entities.keys().toList()
      ..sort((a, b) => a.index.compareTo(b.index));

    if (allKeys.isEmpty) {
      return sb.toString().trimRight();
    }

    // Roots are entities whose parent is null (or missing).
    final roots = <Key>[];
    for (final key in allKeys) {
      final parent = _parents.get(key)?.key;
      if (parent == null || !_entities.contains(parent)) {
        roots.add(key);
      }
    }
    roots.sort((a, b) => a.index.compareTo(b.index));

    void writeSubtree(Key key, String indent, Set<Key> visited) {
      if (!visited.add(key)) {
        sb.writeln('$indent- $key (cycle)');
        return;
      }

      final kids = _children.get(key) ?? const <Key>[];
      sb.writeln('$indent- $key (${kids.length} children)');

      for (final child in kids) {
        writeSubtree(child, '$indent  ', visited);
      }
    }

    final visited = <Key>{};

    sb.writeln('tree:');
    for (final root in roots) {
      writeSubtree(root, '  ', visited);
    }

    // If parent links aren't maintained yet, everything may appear as a root.
    // Still, any unreachable nodes (from roots) or cycles get printed here.
    final remaining = allKeys.where((k) => !visited.contains(k)).toList();
    if (remaining.isNotEmpty) {
      sb.writeln('orphans:');
      for (final k in remaining) {
        writeSubtree(k, '  ', visited);
      }
    }

    return sb.toString().trimRight();
  }

  /// Iterate over all entities in the world in no particular order.
  Iterable<Entity> iter() {
    return _entities.iter();
  }

  /// Traverse the entity tree starting at [root] (pre-order DFS).
  ///
  /// Missing entities/children are skipped defensively. Cycles are guarded
  /// against via a visited set.
  Iterable<Entity> traverse(Key root) sync* {
    final stack = <Key>[root];
    final visited = <Key>{};

    while (stack.isNotEmpty) {
      final key = stack.removeLast();
      if (!visited.add(key)) continue;

      final entity = _entities.get(key);
      if (entity == null) continue;

      yield entity;

      final kids = _children.get(key);
      if (kids == null || kids.isEmpty) continue;

      // Push in reverse so we visit in list order.
      for (var i = kids.length - 1; i >= 0; i--) {
        stack.add(kids[i]);
      }
    }
  }
}
