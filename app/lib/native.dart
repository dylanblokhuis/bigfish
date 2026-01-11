import 'dart:nativewrappers';

base class Window extends NativeFieldWrapperClass1 {
  Window({required int width, required int height, required String title}) {
    createWindow(width, height, title);
  }

  @pragma('vm:external-name', 'create_window')
  external void createWindow(int width, int height, String title);

  @pragma('vm:external-name', 'on_update')
  external void onUpdate(void Function() callback);

  @pragma('vm:external-name', 'on_present')
  external void onPresent(void Function(double interpolation) callback);

  @pragma('vm:external-name', 'poll')
  external bool poll();
}
