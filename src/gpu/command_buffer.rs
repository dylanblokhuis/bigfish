use bigfish_macros::native_impl;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTL4CommandEncoder, MTL4RenderCommandEncoder, MTL4RenderPassDescriptor, MTLBlendFactor,
    MTLColorWriteMask, MTLLoadAction, MTLStoreAction,
};
use objc2_quartz_core::CAMetalDrawable;

use crate::dart_api::{List, NativeArguments, Scope};

use super::{
    compute_encoder::ComputeCommandEncoder, render_encoder::RenderCommandEncoder,
    resources::Texture, gpu::Gpu,
};

type Id<T> = Retained<ProtocolObject<T>>;

pub struct CommandBuffer {
    pub drawable: Id<dyn CAMetalDrawable>,
}

#[native_impl]
impl CommandBuffer {
    fn render_command_encoder(args: NativeArguments, scope: Scope<'_>) {
        let command_buffer_instance = args.get_arg(0).unwrap();
        let _command_buffer = command_buffer_instance.get_peer::<CommandBuffer>().unwrap();
        let gpu_handle = command_buffer_instance
            .get_field(scope.new_string("gpu").unwrap())
            .unwrap();
        let gpu = gpu_handle.get_peer::<Gpu>().unwrap();
        let descriptor_instance = args.get_arg(1).unwrap();
        let descriptor_map = descriptor_instance
            .invoke(scope.new_string("toMap").unwrap(), &mut [])
            .unwrap();

        let pass = MTL4RenderPassDescriptor::new();

        // TODO: clean up this mess
        let color_attachments_key = scope.new_string("colorAttachments").unwrap();
        if let Ok(color_attachments_list) = descriptor_map.map_get(&scope, color_attachments_key) {
            let list_obj = List::new(color_attachments_list).unwrap();
            if let Ok(len) = list_obj.len() {
                for i in 0..(len as usize) {
                    if let Ok(ca_map) = list_obj.get(&scope, i as isize) {
                        let ca = unsafe { pass.colorAttachments().objectAtIndexedSubscript(i) };

                        let texture_key = scope.new_string("texture").unwrap();
                        let texture = ca_map
                            .map_get(&scope, texture_key)
                            .map(|h| h.get_peer::<Texture>().unwrap().texture.as_ref())
                            .ok();
                        ca.setTexture(texture);

                        // Extract load action
                        let load_action_key = scope.new_string("loadAction").unwrap();
                        if let Ok(load_action_handle) = ca_map.map_get(&scope, load_action_key) {
                            if let Ok(load_action_val) = load_action_handle.to_i64() {
                                ca.setLoadAction(MTLLoadAction(load_action_val as usize));
                            }
                        }

                        // Extract store action
                        let store_action_key = scope.new_string("storeAction").unwrap();
                        if let Ok(store_action_handle) = ca_map.map_get(&scope, store_action_key) {
                            if let Ok(store_action_val) = store_action_handle.to_i64() {
                                ca.setStoreAction(MTLStoreAction(store_action_val as usize));
                            }
                        }

                        // Extract clear color (optional)
                        let clear_color_key = scope.new_string("clearColor").unwrap();
                        if let Ok(clear_color_list) = ca_map.map_get(&scope, clear_color_key) {
                            let clear_color_list_obj = List::new(clear_color_list).unwrap();
                            if let Ok(clear_color_len) = clear_color_list_obj.len() {
                                if clear_color_len >= 4 {
                                    if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                                        clear_color_list_obj
                                            .get(&scope, 0)
                                            .and_then(|h| h.to_f64()),
                                        clear_color_list_obj
                                            .get(&scope, 1)
                                            .and_then(|h| h.to_f64()),
                                        clear_color_list_obj
                                            .get(&scope, 2)
                                            .and_then(|h| h.to_f64()),
                                        clear_color_list_obj
                                            .get(&scope, 3)
                                            .and_then(|h| h.to_f64()),
                                    ) {
                                        ca.setClearColor(objc2_metal::MTLClearColor {
                                            red: r,
                                            green: g,
                                            blue: b,
                                            alpha: a,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let render_command_encoder = gpu
            .command_buffer
            .renderCommandEncoderWithDescriptor(&pass)
            .unwrap();
        let render_command_encoder_instance = scope
            .new_object(
                scope
                    .get_class(
                        scope.library("package:app/native.dart").unwrap(),
                        "RenderCommandEncoder",
                    )
                    .unwrap(),
                scope.null_handle().unwrap(),
                &mut [],
            )
            .unwrap();
        render_command_encoder_instance
            .set_peer(Box::new(RenderCommandEncoder(render_command_encoder)));
        args.set_return_value(render_command_encoder_instance);
    }

    fn compute_command_encoder(args: NativeArguments, scope: Scope<'_>) {
        let command_buffer_instance = args.get_arg(0).unwrap();
        let _command_buffer = command_buffer_instance.get_peer::<CommandBuffer>().unwrap();
        let gpu_handle = command_buffer_instance
            .get_field(scope.new_string("gpu").unwrap())
            .unwrap();
        let gpu = gpu_handle.get_peer::<Gpu>().unwrap();

        let compute_command_encoder = gpu.command_buffer.computeCommandEncoder().unwrap();
        let compute_command_encoder_instance = scope
            .new_object(
                scope
                    .get_class(
                        scope.library("package:app/native.dart").unwrap(),
                        "ComputeCommandEncoder",
                    )
                    .unwrap(),
                scope.null_handle().unwrap(),
                &mut [],
            )
            .unwrap();
        compute_command_encoder_instance
            .set_peer(Box::new(ComputeCommandEncoder(compute_command_encoder)));
        args.set_return_value(compute_command_encoder_instance);
    }

    fn drawable(args: NativeArguments, scope: Scope<'_>) {
        let command_buffer_instance = args.get_arg(0).unwrap();
        let command_buffer = command_buffer_instance.get_peer::<CommandBuffer>().unwrap();
        let texture = command_buffer.drawable.texture();

        let library = scope.library("package:app/native.dart").unwrap();
        let class_type = scope.get_class(library, "Texture").unwrap();
        let class_instance = scope
            .new_object(class_type, scope.null_handle().unwrap(), &mut [])
            .unwrap();
        class_instance.set_peer(Box::new(Texture { texture }));
        args.set_return_value(class_instance);
    }
}
