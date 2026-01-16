import 'dart:typed_data';
import 'dart:math' as math;

import 'package:app/world.dart';

import 'native.dart';

class SimpleRaytracer {
  static const int outputWidth = 800;
  static const int outputHeight = 600;

  late ComputePipeline computePipeline;
  late RenderPipeline blitPipeline;
  late Buffer vertexBuffer;
  late Buffer scratchBuffer;
  late AccelerationStructure accelerationStructure;
  late AccelerationStructureDescriptor accelerationStructureDescriptor;
  late BufferRange scratchBufferRange;
  late Texture colorTexture;
  late ArgumentTable argumentTable;
  late ArgumentTable blitArgumentTable;

  SimpleRaytracer(Gpu gpu) {
    computePipeline = gpu.compileComputePipeline(
      ComputePipelineDescriptor(
        computeShader: ShaderLibrary(
          path: "./app/shaders/compute.slang",
          entryPoint: "computeShader",
        ),
      ),
    );

    vertexBuffer = gpu.createBuffer(3 * 4 * 4);
    vertexBuffer.setContents(_trianglePositionsBytes(0.0));
    gpu.addBufferToResidencySet(vertexBuffer);

    final triangleDescriptor = TriangleGeometryDescriptor(
      vertexBuffer: BufferRange.fromBuffer(
        vertexBuffer,
        length: vertexBuffer.length(),
      ),
      triangleCount: 1,
      vertexStride: 4 * 4,
      vertexFormat: VertexFormat.float4,
    );
    accelerationStructureDescriptor = PrimitiveAccelerationStructureDescriptor(
      geometryDescriptors: [triangleDescriptor],
    );

    final accelerationSizes = gpu.accelerationStructureSizes(
      accelerationStructureDescriptor,
    );
    accelerationStructure = gpu.createAccelerationStructure(
      accelerationSizes.accelerationStructureSize,
    );
    scratchBuffer = gpu.createBuffer(accelerationSizes.buildScratchBufferSize);
    scratchBufferRange = BufferRange.fromBuffer(
      scratchBuffer,
      length: accelerationSizes.buildScratchBufferSize,
    );

    colorTexture = gpu.createTexture(
      outputWidth,
      outputHeight,
      PixelFormat.rgba8Unorm.value,
    );
    gpu.addTextureToResidencySet(colorTexture);
    gpu.addBufferToResidencySet(scratchBuffer);
    gpu.addAccelerationStructureToResidencySet(accelerationStructure);

    // Make residency additions visible to the GPU.
    gpu.commitResidencySet();

    // Create the argument table and bind GPU addresses (buffer indices in shader).
    argumentTable = gpu.createArgumentTable(
      maxBufferBindCount: 2,
      maxTextureBindCount: 1,
    );
    argumentTable.setTexture(colorTexture, 0);
    argumentTable.setAccelerationStructure(accelerationStructure, 0);

    // Create blit render pipeline
    blitPipeline = gpu.compileRenderPipeline(
      RenderPipelineDescriptor(
        colorAttachments: [
          RenderPipelineDescriptorColorAttachment(
            pixelFormat: PixelFormat.bgra8Unorm,
          ),
        ],
        vertexShader: ShaderLibrary(
          path: "./app/shaders/blit.slang",
          entryPoint: "vertexShader",
        ),
        fragmentShader: ShaderLibrary(
          path: "./app/shaders/blit.slang",
          entryPoint: "fragmentShader",
        ),
        primitiveTopology: PrimitiveTopology.triangle,
      ),
    );

    // Create argument table for blit pipeline
    blitArgumentTable = gpu.createArgumentTable(
      maxBufferBindCount: 0,
      maxTextureBindCount: 1,
      maxSamplerStateBindCount: 1,
    );
    blitArgumentTable.setTexture(colorTexture, 0);

    final linearSampler = gpu.createSampler(
      SamplerDescriptor(
        minFilter: SamplerMinMagFilter.linear,
        magFilter: SamplerMinMagFilter.linear,
        mipFilter: SamplerMipFilter.linear,
        addressModeU: SamplerAddressMode.clampToEdge,
        addressModeV: SamplerAddressMode.clampToEdge,
        addressModeW: SamplerAddressMode.clampToEdge,
      ),
    );
    blitArgumentTable.setSampler(linearSampler, 0);
  }
}

void main() {
  final window = Window(width: 800, height: 600, title: 'Hello World');
  final gpu = Gpu(window);

  final world = World();
  world.insertResource(SimpleRaytracer(gpu));
  window.onUpdate(() => update(world));
  window.onPresent((interpolation) => present(world, gpu, interpolation));

  while (window.poll()) {}
}

// update game logic at 60 ticks
void update(World world) {
  world.spawn();
}

// we can render here, will loop as fast as possible, with the interpolation value being the amount of time that has passed since the last update
// can be used to interpolate values to not have janky movement
void present(World world, Gpu gpu, double interpolation) {
  final raytracer = world.getResource<SimpleRaytracer>();
  final commandBuffer = gpu.beginCommandBuffer();

  final computeCommandEncoder = commandBuffer.computeCommandEncoder();
  // Rotate the triangle and rebuild the acceleration structure.
  final nowMs = DateTime.now().millisecondsSinceEpoch;
  final rotationDegrees = (nowMs / 1000.0) * 60.0; // 60 deg/sec
  raytracer.vertexBuffer.setContents(_trianglePositionsBytes(rotationDegrees));
  computeCommandEncoder.buildAccelerationStructure(
    accelerationStructure: raytracer.accelerationStructure,
    descriptor: raytracer.accelerationStructureDescriptor,
    scratchBufferRange: raytracer.scratchBufferRange,
  );
  computeCommandEncoder.intraPassBarrier(
    afterEncoderStages: GpuStage.accelerationStructure,
    beforeEncoderStages: GpuStage.dispatch,
    visibilityOptions: VisibilityOptions.device,
  );

  computeCommandEncoder.setComputePipeline(raytracer.computePipeline);
  computeCommandEncoder.setArgumentTable(raytracer.argumentTable);
  computeCommandEncoder.dispatchThreads(
    SimpleRaytracer.outputWidth,
    SimpleRaytracer.outputHeight,
    1,
    8,
    8,
    1,
  );
  computeCommandEncoder.intraPassBarrier(
    afterEncoderStages: GpuStage.dispatch,
    beforeEncoderStages: GpuStage.blit,
    visibilityOptions: VisibilityOptions.device,
  );

  computeCommandEncoder.endEncoding();

  // blit using a render command encoder
  final drawable = commandBuffer.drawable();
  final renderCommandEncoder = commandBuffer.renderCommandEncoder(
    RenderPassDescriptor(
      colorAttachments: [
        RenderPassDescriptorColorAttachment(
          texture: drawable,
          loadAction: LoadAction.clear,
          storeAction: StoreAction.store,
        ),
      ],
    ),
  );

  // Wait for compute shader to finish writing to colorTexture
  renderCommandEncoder.consumerBarrier(
    afterStages: GpuStage.dispatch,
    beforeStages: GpuStage.fragment,
    visibilityOptions: VisibilityOptions.device,
  );

  renderCommandEncoder.setRenderPipeline(raytracer.blitPipeline);
  renderCommandEncoder.setViewport(
    width: SimpleRaytracer.outputWidth.toDouble(),
    height: SimpleRaytracer.outputHeight.toDouble(),
  );
  renderCommandEncoder.setScissorRect(
    width: SimpleRaytracer.outputWidth,
    height: SimpleRaytracer.outputHeight,
  );
  renderCommandEncoder.setArgumentTable(raytracer.blitArgumentTable);
  renderCommandEncoder.drawPrimitives(
    primitiveType: PrimitiveType.triangle,
    vertexCount: 3,
    instanceCount: 1,
  );
  renderCommandEncoder.endEncoding();

  gpu.endCommandBuffer(commandBuffer);
}

Uint8List _trianglePositionsBytes(double rotationDegrees) {
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
    // v1
    x1, y1, 0.0, 1.0,
    // v2
    x2, y2, 0.0, 1.0,
  ];

  final bd = ByteData(floats.length * 4);
  for (var i = 0; i < floats.length; i++) {
    bd.setFloat32(i * 4, floats[i].toDouble(), Endian.little);
  }
  return bd.buffer.asUint8List();
}
