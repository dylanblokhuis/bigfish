import 'native.dart';

void main() {
  final window = Window(width: 800, height: 600, title: 'Hello World');

  window.onUpdate(() => update());
  window.onPresent((interpolation) => present(interpolation));

  // we poll here because if we use a long lived function, we cant hot reload anything
  while (window.poll()) {}
}

// update game logic at 60 ticks
void update() {
  print('Update!');
}

// we can render here, will loop as fast as possible, with the interpolation value being the amount of time that has passed since the last update
// can be used to interpolate values to not have janky movement
void present(double interpolation) {
  print("Present! $interpolation");
}
