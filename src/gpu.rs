use bigfish_macros::native_func;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTL4ArgumentTable, MTL4ArgumentTableDescriptor, MTL4CommandAllocator, MTL4CommandBuffer,
    MTL4CommandQueue, MTL4CompilerDescriptor, MTL4RenderPassDescriptor,
    MTL4RenderPipelineDescriptor, MTLCreateSystemDefaultDevice, MTLDevice, MTLEvent, MTLLoadAction,
    MTLPixelFormat, MTLPrimitiveType, MTLRenderPipelineState, MTLRenderStages, MTLResidencySet,
    MTLResidencySetDescriptor, MTLSharedEvent, MTLStoreAction, MTLViewport,
};
// Bring ObjC protocol traits into scope for method resolution.
use objc2_metal::{
    MTL4CommandEncoder as _, MTL4Compiler as _, MTL4RenderCommandEncoder as _, MTLBuffer as _,
    MTLDrawable as _,
};
use objc2_quartz_core::CAMetalDrawable;

use crate::dart_api::{sys, Isolate, NativeArguments};
use crate::window::Window;

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    position: [f32; 4],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct TriangleData {
    vertices: [Vertex; 3],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct ViewportSize {
    size: [u32; 2],
}

type Id<T> = Retained<ProtocolObject<T>>;

struct Gpu {
    device: Id<dyn MTLDevice>,
    command_queue: Id<dyn MTL4CommandQueue>,
    command_buffer: Id<dyn MTL4CommandBuffer>,
    command_allocators: Vec<Id<dyn MTL4CommandAllocator>>,
    residency_set: Id<dyn MTLResidencySet>,
    argument_table: Id<dyn MTL4ArgumentTable>,
    render_pipeline_state: Id<dyn MTLRenderPipelineState>,
    triangle_vertex_buffers: Vec<Id<dyn objc2_metal::MTLBuffer>>,
    viewport_size_buffer: Id<dyn objc2_metal::MTLBuffer>,
    shared_event: Id<dyn MTLSharedEvent>,
    frame_number: u64,
    window_peer: *mut Window,
}

#[native_func]
fn init_gpu(args: NativeArguments) {
    let instance = args.get_arg(0).unwrap();
    let window_handle = args.get_arg(1).unwrap();
    let window_peer = window_handle.get_peer::<Window>().unwrap() as *mut Window;

    let frames_in_flight = 3;
    let device = MTLCreateSystemDefaultDevice().unwrap();
    let command_queue = device.newMTL4CommandQueue().unwrap();
    let command_buffer = device.newCommandBuffer().unwrap();
    let mut command_allocators = Vec::with_capacity(frames_in_flight);
    for _ in 0..frames_in_flight {
        let command_allocator = device.newCommandAllocator().unwrap();
        command_allocators.push(command_allocator);
    }
    let desc = MTLResidencySetDescriptor::new();
    let residency_set = device.newResidencySetWithDescriptor_error(&desc).unwrap();

    // Bind the SDL-created CAMetalLayer to this device and configure basics.
    #[cfg(target_os = "macos")]
    {
        use objc2_quartz_core::CAMetalLayer;
        let window = unsafe { &*window_peer };
        let layer: &CAMetalLayer = window.metal_layer();
        layer.setDevice(Some(device.as_ref()));
        layer.setPixelFormat(MTLPixelFormat::BGRA8Unorm);
        layer.setMaximumDrawableCount(frames_in_flight as usize);
    }

    // Argument table: two GPU addresses (triangle data + viewport size).
    let table_desc = MTL4ArgumentTableDescriptor::new();
    table_desc.setMaxBufferBindCount(2);
    let argument_table = device
        .newArgumentTableWithDescriptor_error(&table_desc)
        .unwrap();

    // Buffers.
    let mut triangle_vertex_buffers = Vec::with_capacity(frames_in_flight);
    for _ in 0..frames_in_flight {
        let buf = device
            .newBufferWithLength_options(
                core::mem::size_of::<TriangleData>() as usize,
                objc2_metal::MTLResourceOptions::StorageModeShared,
            )
            .unwrap();
        triangle_vertex_buffers.push(buf);
    }

    let viewport_size_buffer = device
        .newBufferWithLength_options(
            core::mem::size_of::<ViewportSize>() as usize,
            objc2_metal::MTLResourceOptions::StorageModeShared,
        )
        .unwrap();

    // Add long-lived buffers to residency set and attach sets to queue.
    residency_set.addAllocation(viewport_size_buffer.as_ref());
    for b in &triangle_vertex_buffers {
        residency_set.addAllocation(b.as_ref());
    }
    residency_set.commit();
    command_queue.addResidencySet(&residency_set);

    #[cfg(target_os = "macos")]
    {
        let window = unsafe { &*window_peer };
        command_queue.addResidencySet(&window.metal_layer().residencySet());
    }

    // Compile pipeline via Metal 4 compiler.
    let compiler_desc = MTL4CompilerDescriptor::new();
    let compiler = device
        .newCompilerWithDescriptor_error(&compiler_desc)
        .unwrap();

    let shader_src = objc2_foundation::NSString::from_str(
        r#"
#include <metal_stdlib>
using namespace metal;

struct Vertex { float4 position; float4 color; };
struct TriangleData { Vertex vertices[3]; };
struct ViewportSize { uint2 size; };

struct Args {
    device TriangleData* tri [[id(0)]];
    device ViewportSize* viewport [[id(1)]];
} [[argument_table]];

struct VSOut { float4 position [[position]]; float4 color; };

vertex VSOut vertexShader(uint vid [[vertex_id]], Args args [[argument_table]]) {
    VSOut out;
    Vertex v = args.tri->vertices[vid];
    out.position = v.position;
    out.color = v.color;
    return out;
}

fragment float4 fragmentShader(VSOut in [[stage_in]]) {
    return in.color;
}
"#,
    );

    let library = device
        .newLibraryWithSource_options_error(&shader_src, None)
        .unwrap();

    let vfd = objc2_metal::MTL4LibraryFunctionDescriptor::new();
    vfd.setLibrary(Some(&library));
    vfd.setName(Some(&objc2_foundation::NSString::from_str("vertexShader")));

    let ffd = objc2_metal::MTL4LibraryFunctionDescriptor::new();
    ffd.setLibrary(Some(&library));
    ffd.setName(Some(&objc2_foundation::NSString::from_str(
        "fragmentShader",
    )));

    let rp_desc = MTL4RenderPipelineDescriptor::new();
    rp_desc.setVertexFunctionDescriptor(Some(&*vfd));
    rp_desc.setFragmentFunctionDescriptor(Some(&*ffd));
    unsafe {
        rp_desc
            .colorAttachments()
            .objectAtIndexedSubscript(0)
            .setPixelFormat(MTLPixelFormat::BGRA8Unorm);
    }

    let render_pipeline_state = compiler
        .newRenderPipelineStateWithDescriptor_compilerTaskOptions_error(&rp_desc, None)
        .unwrap();

    let shared_event = device.newSharedEvent().unwrap();
    shared_event.setSignaledValue(0);

    instance.set_peer(Box::new(Gpu {
        device,
        command_queue,
        command_buffer,
        command_allocators,
        residency_set,
        argument_table,
        render_pipeline_state,
        triangle_vertex_buffers,
        viewport_size_buffer,
        shared_event,
        frame_number: 0,
        window_peer,
    }));
}

struct CommandBuffer {
    drawable: Id<dyn CAMetalDrawable>,
}

#[native_func]
fn begin_command_buffer(args: NativeArguments) {
    let gpu_instance = args.get_arg(0).unwrap();
    let gpu = gpu_instance.get_peer::<Gpu>().unwrap();
    let window = unsafe { &*gpu.window_peer };

    let drawable = match window.metal_layer().nextDrawable() {
        Some(d) => d,
        None => return,
    };

    gpu.frame_number += 1;
    let frame_index = (gpu.frame_number as usize) % gpu.command_allocators.len();

    if gpu.frame_number > gpu.command_allocators.len() as u64 {
        let earlier = gpu.frame_number - gpu.command_allocators.len() as u64;
        let _timed_out = gpu
            .shared_event
            .waitUntilSignaledValue_timeoutMS(earlier, 10);
    }

    let allocator = &gpu.command_allocators[frame_index];
    allocator.reset();

    gpu.command_buffer
        .beginCommandBufferWithAllocator(allocator);
    gpu.command_buffer.useResidencySet(&gpu.residency_set);

    let scope = Isolate::current().unwrap();
    let library = scope.library("package:app/native.dart").unwrap();
    let class_type = scope.get_class(library, "CommandBuffer").unwrap();
    // let constructor_name = scope.new_string("CommandBuffer").unwrap();
    let class_instance = scope
        .new_object(class_type, scope.null_handle().unwrap(), &mut [])
        .unwrap();
    class_instance.set_peer(Box::new(CommandBuffer { drawable }));
    class_instance.set_field(scope.new_string("gpu").unwrap(), &gpu_instance);
    args.set_return_value(class_instance);
}

#[native_func]
fn end_command_buffer(args: NativeArguments) {
    let gpu_instance = args.get_arg(0).unwrap();
    let gpu = gpu_instance.get_peer::<Gpu>().unwrap();
    let command_buffer_instance = args.get_arg(1).unwrap();
    let command_buffer = command_buffer_instance.get_peer::<CommandBuffer>().unwrap();

    gpu.command_buffer.endCommandBuffer();

    // Submit + present (Metal 4 queue semantics).
    let drawable_mtl: &ProtocolObject<dyn objc2_metal::MTLDrawable> =
        command_buffer.drawable.as_ref();
    gpu.command_queue.waitForDrawable(drawable_mtl);

    let buf_ptr = core::ptr::NonNull::from(&*gpu.command_buffer);
    let mut bufs = [buf_ptr];
    unsafe {
        gpu.command_queue
            .commit_count(core::ptr::NonNull::new(bufs.as_mut_ptr()).unwrap(), 1);
    }

    gpu.command_queue.signalDrawable(drawable_mtl);
    command_buffer.drawable.present();

    let event: &ProtocolObject<dyn MTLEvent> = gpu.shared_event.as_ref();
    gpu.command_queue.signalEvent_value(event, gpu.frame_number);
}

#[native_func]
fn gpu_draw(args: NativeArguments) {
    let instance = args.get_arg(0).unwrap();
    let gpu = instance.get_peer::<Gpu>().unwrap();

    // Safety: we assume the Window outlives the Gpu (both are kept alive by Dart).
    let window = unsafe { &*gpu.window_peer };

    gpu.frame_number += 1;
    let frame_index = (gpu.frame_number as usize) % gpu.command_allocators.len();

    if gpu.frame_number > gpu.command_allocators.len() as u64 {
        let earlier = gpu.frame_number - gpu.command_allocators.len() as u64;
        let _timed_out = gpu
            .shared_event
            .waitUntilSignaledValue_timeoutMS(earlier, 10);
    }

    let allocator = &gpu.command_allocators[frame_index];
    allocator.reset();

    gpu.command_buffer
        .beginCommandBufferWithAllocator(allocator);
    gpu.command_buffer.useResidencySet(&gpu.residency_set);

    #[cfg(target_os = "macos")]
    let drawable = match window.metal_layer().nextDrawable() {
        Some(d) => d,
        None => return,
    };

    #[cfg(target_os = "macos")]
    {
        // Update viewport buffer.
        let (w, h) = window.size();
        let vp = ViewportSize {
            size: [w as u32, h as u32],
        };
        unsafe {
            let dst = gpu.viewport_size_buffer.contents().as_ptr() as *mut ViewportSize;
            dst.write(vp);
        }

        // Update triangle buffer.
        let rotation = gpu.frame_number as f32;
        let tri = calculate_triangle(rotation);
        unsafe {
            let dst =
                gpu.triangle_vertex_buffers[frame_index].contents().as_ptr() as *mut TriangleData;
            dst.write(tri);
        }

        // Build render pass.
        let pass = MTL4RenderPassDescriptor::new();
        let ca0 = unsafe { pass.colorAttachments().objectAtIndexedSubscript(0) };
        let tex = drawable.texture();
        ca0.setTexture(Some(tex.as_ref()));
        ca0.setLoadAction(MTLLoadAction::Clear);
        ca0.setStoreAction(MTLStoreAction::Store);
        ca0.setClearColor(objc2_metal::MTLClearColor {
            red: 0.05,
            green: 0.05,
            blue: 0.08,
            alpha: 1.0,
        });

        let encoder = match gpu.command_buffer.renderCommandEncoderWithDescriptor(&pass) {
            Some(e) => e,
            None => {
                gpu.command_buffer.endCommandBuffer();
                return;
            }
        };

        encoder.setRenderPipelineState(&gpu.render_pipeline_state);
        encoder.setViewport(MTLViewport {
            originX: 0.0,
            originY: 0.0,
            width: w as f64,
            height: h as f64,
            znear: 0.0,
            zfar: 1.0,
        });

        unsafe {
            gpu.argument_table
                .setAddress_atIndex(gpu.triangle_vertex_buffers[frame_index].gpuAddress(), 0);
            gpu.argument_table
                .setAddress_atIndex(gpu.viewport_size_buffer.gpuAddress(), 1);
        }
        encoder.setArgumentTable_atStages(&gpu.argument_table, MTLRenderStages::Vertex);

        unsafe {
            encoder.drawPrimitives_vertexStart_vertexCount(MTLPrimitiveType::Triangle, 0, 3);
        }
        encoder.endEncoding();

        gpu.command_buffer.endCommandBuffer();

        // Submit + present (Metal 4 queue semantics).
        let drawable_mtl: &ProtocolObject<dyn objc2_metal::MTLDrawable> = drawable.as_ref();
        gpu.command_queue.waitForDrawable(drawable_mtl);

        let buf_ptr = core::ptr::NonNull::from(&*gpu.command_buffer);
        let mut bufs = [buf_ptr];
        unsafe {
            gpu.command_queue
                .commit_count(core::ptr::NonNull::new(bufs.as_mut_ptr()).unwrap(), 1);
        }

        gpu.command_queue.signalDrawable(drawable_mtl);
        drawable.present();

        let event: &ProtocolObject<dyn MTLEvent> = gpu.shared_event.as_ref();
        gpu.command_queue.signalEvent_value(event, gpu.frame_number);
    }
}

fn calculate_triangle(rotation_degrees: f32) -> TriangleData {
    let radius = 0.5_f32;
    let angle = rotation_degrees * core::f32::consts::PI / 180.0;

    let v0 = Vertex {
        position: [radius * angle.cos(), radius * angle.sin(), 0.0, 1.0],
        color: [1.0, 0.0, 0.0, 1.0],
    };
    let v1 = Vertex {
        position: [
            radius * (angle + 2.0 * core::f32::consts::PI / 3.0).cos(),
            radius * (angle + 2.0 * core::f32::consts::PI / 3.0).sin(),
            0.0,
            1.0,
        ],
        color: [0.0, 1.0, 0.0, 1.0],
    };
    let v2 = Vertex {
        position: [
            radius * (angle + 4.0 * core::f32::consts::PI / 3.0).cos(),
            radius * (angle + 4.0 * core::f32::consts::PI / 3.0).sin(),
            0.0,
            1.0,
        ],
        color: [0.0, 0.0, 1.0, 1.0],
    };

    TriangleData {
        vertices: [v0, v1, v2],
    }
}
