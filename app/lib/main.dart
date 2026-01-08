import 'dart:ffi';
import 'dart:nativewrappers';

void main() {
  print('Hello world');
}

@pragma("vm:entry-point")
void tick() {
  final here = MyClass();
  final yo = MyClass.hello();
}

base class MyClass extends NativeFieldWrapperClass1 {
  MyClass();

  @pragma('vm:external-name', 'hello')
  external static String hello();
}
