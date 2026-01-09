import 'dart:nativewrappers';

void main() {
  final window = Window();
  window.createWindow(800, 600, 'Hello World');

  // Set up update and present callbacks
  window.setUpdateCallback(update);

  window.setPresentCallback(present);

  while (window.poll()) {}
}

void update() {
  print('Update');
}

void present() {
  print("Present!");
}

base class MyClass extends NativeFieldWrapperClass1 {
  MyClass();

  @pragma('vm:external-name', 'init_gpu')
  external static void initGpu(bool a);
}

base class Window extends NativeFieldWrapperClass1 {
  Window();

  @pragma('vm:external-name', 'create_window')
  external void createWindow(int width, int height, String title);

  @pragma('vm:external-name', 'set_update_callback')
  external void setUpdateCallback(void Function() callback);

  @pragma('vm:external-name', 'set_present_callback')
  external void setPresentCallback(void Function() callback);

  @pragma('vm:external-name', 'poll')
  external bool poll();
}
