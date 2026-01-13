import 'dart:io';
import 'dart:typed_data';
import 'dart:math' as math;

import 'package:app/world.dart';

import 'native.dart';

class SimpleRaster {
  late RenderPipeline renderPipeline;
  late Buffer vertexBuffer;
  late Buffer viewportBuffer;

  SimpleRaster(Gpu gpu) {
    final metalShader = File("./app/shaders/Shaders.metal").readAsStringSync();
    final descriptor = RenderPipelineDescriptor(
      colorAttachments: [
        RenderPipelineDescriptorColorAttachment(
          pixelFormat: PixelFormat.bgra8Unorm,
        ),
      ],
      vertexShader: ShaderLibrary(
        source: metalShader,
        entryPoint: 'vertexShader',
      ),
      fragmentShader: ShaderLibrary(
        source: metalShader,
        entryPoint: 'fragmentShader',
      ),
      primitiveTopology: PrimitiveTopology.triangle,
    );
    renderPipeline = gpu.compileRenderPipeline(descriptor);

    // Vertex buffer: 3 vertices, each vertex is float4 position + float4 color.
    // Layout must match `struct Vertex` in `app/shaders/Shaders.metal`.
    vertexBuffer = gpu.createBuffer(3 * 8 * 4);
    vertexBuffer.setContents(_triangleVerticesBytes(0.0));
    gpu.addBufferToResidencySet(vertexBuffer);

    // Viewport buffer: uint2 (width, height) used by the vertex shader.
    viewportBuffer = gpu.createBuffer(2 * 4);
    viewportBuffer.setContents(_viewportBytes(width: 800, height: 600));
    gpu.addBufferToResidencySet(viewportBuffer);

    // Make residency additions visible to the GPU.
    gpu.commitResidencySet();

    // Initialize the argument table bindings (buffer indices in shader).
    gpu.setBufferInArgumentTable(vertexBuffer, 0);
    gpu.setBufferInArgumentTable(viewportBuffer, 1);
  }
}

void main() {
  final window = Window(width: 800, height: 600, title: 'Hello World');
  final gpu = Gpu(window);

  final world = World();
  world.insertResource(SimpleRaster(gpu));
  window.onUpdate(() => update(world));
  window.onPresent((interpolation) => present(world, gpu, interpolation));

  while (window.poll()) {}
}

// update game logic at 60 ticks
void update(World world) {
  // print(time);
  world.spawn();

  // final child = world.spawn();
  // world.addChild(root, child);

  // print(world);
}

// we can render here, will loop as fast as possible, with the interpolation value being the amount of time that has passed since the last update
// can be used to interpolate values to not have janky movement
void present(World world, Gpu gpu, double interpolation) {
  final simpleRaster = world.getResource<SimpleRaster>();
  // print("Present! $interpolation");
  final commandBuffer = gpu.beginCommandBuffer();
  final renderCommandEncoder = commandBuffer.renderCommandEncoder();
  renderCommandEncoder.setRenderPipeline(simpleRaster.renderPipeline);

  // Animate the triangle like the old Rust example.
  final nowMs = DateTime.now().millisecondsSinceEpoch;
  final rotationDegrees = (nowMs / 1000.0) * 60.0; // 60 deg/sec
  simpleRaster.vertexBuffer.setContents(
    _triangleVerticesBytes(rotationDegrees),
  );

  renderCommandEncoder.setArgumentTable(gpu);
  renderCommandEncoder.setViewport(width: 800, height: 600);
  renderCommandEncoder.drawPrimitives(
    primitiveType: PrimitiveType.triangle,
    vertexCount: 3,
    instanceCount: 1,
  );
  renderCommandEncoder.endEncoding();
  gpu.endCommandBuffer(commandBuffer);
}

Uint8List _triangleVerticesBytes(double rotationDegrees) {
  final radius = 0.5;
  final angle = rotationDegrees * math.pi / 180.0;

  double x0 = radius * math.cos(angle);
  double y0 = radius * math.sin(angle);
  double x1 = radius * math.cos(angle + 2.0 * math.pi / 3.0);
  double y1 = radius * math.sin(angle + 2.0 * math.pi / 3.0);
  double x2 = radius * math.cos(angle + 4.0 * math.pi / 3.0);
  double y2 = radius * math.sin(angle + 4.0 * math.pi / 3.0);

  final floats = <double>[
    // v0
    x0, y0, 0.0, 1.0,
    1.0, 0.0, 0.0, 1.0,
    // v1
    x1, y1, 0.0, 1.0,
    0.0, 1.0, 0.0, 1.0,
    // v2
    x2, y2, 0.0, 1.0,
    0.0, 0.0, 1.0, 1.0,
  ];

  final bd = ByteData(floats.length * 4);
  for (var i = 0; i < floats.length; i++) {
    bd.setFloat32(i * 4, floats[i].toDouble(), Endian.little);
  }
  return bd.buffer.asUint8List();
}

Uint8List _viewportBytes({required int width, required int height}) {
  final bd = ByteData(8);
  bd.setUint32(0, width, Endian.little);
  bd.setUint32(4, height, Endian.little);
  return bd.buffer.asUint8List();
}
