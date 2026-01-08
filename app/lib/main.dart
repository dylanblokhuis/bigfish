import 'dart:ffi';
import 'dart:nativewrappers';

void main() {
  print('Hello world');
}

@pragma("vm:entry-point")
void tick() {
  MyClass.initGpu(true);
  print('Tick!!!');
}

base class MyClass extends NativeFieldWrapperClass1 {
  MyClass();

  @pragma('vm:external-name', 'init_gpu')
  external static void initGpu(bool a);
}
