class Slot<T> {
  Slot({this.nextFree, this.value, this.version = 0});
  T? value;
  int? nextFree;
  int version;
}

class Key {
  Key({required this.index, required this.version});
  int index;
  int version;

  // override equals
  @override
  bool operator ==(Object other) {
    if (other is Key) {
      return index == other.index && version == other.version;
    }
    return false;
  }

  @override
  int get hashCode => index ^ version;

  @override
  String toString() {
    return 'Key(${index}v$version)';
  }
}

class SlotMap<T> {
  late List<Slot<T>> _slots;
  late int _freeHead;
  late int _numElements;

  SlotMap() {
    _slots = List.empty(growable: true);
    _slots.add(Slot(nextFree: 0, value: null));
    _freeHead = 1;
    _numElements = 0;
  }

  int length() {
    return _numElements;
  }

  bool isEmpty() {
    return _numElements == 0;
  }

  Key insert(T value) {
    Slot<T>? slot = _slots.elementAtOrNull(_freeHead);
    if (slot != null) {
      slot.value = value;
      Key key = Key(index: _freeHead, version: slot.version);
      _freeHead = slot.nextFree!;
      _numElements++;
      // we found a slot, so we can reuse the array slot instead of growing the list
      return key;
    } else {
      // we didn't find a slot, so we need to grow the list
      _slots.add(Slot(nextFree: 0, value: value));
      _freeHead = _slots.length;
      _numElements++;
      return Key(index: _slots.length - 1, version: 0);
    }
  }

  T? get(Key key) {
    Slot<T>? slot = _slots.elementAtOrNull(key.index);
    if (contains(key)) {
      return slot!.value;
    }
    return null;
  }

  void set(Key key, T value) {
    if (!contains(key)) {
      throw Exception('Key not found');
    }
    _slots[key.index].value = value;
  }

  bool contains(Key key) {
    Slot<T>? slot = _slots.elementAtOrNull(key.index);
    return slot != null && slot.version == key.version;
  }

  T? remove(Key key) {
    if (!contains(key)) {
      return null;
    }

    // swap remove
    final value = _slots[key.index].value;
    _slots[key.index].value = null;
    _slots[key.index].nextFree = _freeHead;
    _freeHead = key.index;
    _numElements--;

    return value!;
  }

  Iterable<T> iter() sync* {
    int cur = 0;
    while (cur < _slots.length) {
      final slot = _slots[cur];
      if (slot.value != null) {
        yield slot.value!;
      }
      cur++;
    }
  }

  /// Iterate over all live keys in this slot map.
  Iterable<Key> keys() sync* {
    for (var i = 0; i < _slots.length; i++) {
      final slot = _slots[i];
      if (slot.value != null) {
        yield Key(index: i, version: slot.version);
      }
    }
  }

  /// Iterate over all live key/value pairs in this slot map.
  Iterable<MapEntry<Key, T>> entries() sync* {
    for (var i = 0; i < _slots.length; i++) {
      final slot = _slots[i];
      final value = slot.value;
      if (value != null) {
        yield MapEntry(Key(index: i, version: slot.version), value);
      }
    }
  }
}
