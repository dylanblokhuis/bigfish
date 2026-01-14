use bigfish_macros::native_impl;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{
    MTL4RenderCommandEncoder, MTL4VisibilityOptions, MTLBlendFactor, MTLColorWriteMask,
    MTLCullMode, MTLPrimitiveType, MTLRenderStages, MTLScissorRect, MTLStages, MTLViewport,
};

use crate::dart_api::{NativeArguments, Scope};

use super::resources::{ArgumentTable, RenderPipeline};

type Id<T> = Retained<ProtocolObject<T>>;

pub struct RenderCommandEncoder(pub Id<dyn MTL4RenderCommandEncoder>);

#[native_impl]
impl RenderCommandEncoder {
    fn set_render_pipeline(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();

        let render_pipeline = args.get_arg(1).unwrap();
        let render_pipeline = render_pipeline.get_peer::<RenderPipeline>().unwrap();

        render_command_encoder
            .0
            .setRenderPipelineState(&render_pipeline.render_pipeline_state);
    }

    fn set_viewport(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();

        let x = args.get_double_arg(1).unwrap();
        let y = args.get_double_arg(2).unwrap();
        let width = args.get_double_arg(3).unwrap();
        let height = args.get_double_arg(4).unwrap();

        render_command_encoder.0.setViewport(MTLViewport {
            originX: x,
            originY: y,
            width,
            height,
            znear: 0.0,
            zfar: 1.0,
        });
    }

    fn set_scissor_rect(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();

        let x = args.get_integer_arg(1).unwrap() as usize;
        let y = args.get_integer_arg(2).unwrap() as usize;
        let width = args.get_integer_arg(3).unwrap() as usize;
        let height = args.get_integer_arg(4).unwrap() as usize;

        render_command_encoder.0.setScissorRect(MTLScissorRect {
            x,
            y,
            width,
            height,
        });
    }

    fn set_cull_mode(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();

        let mode = args.get_integer_arg(1).unwrap() as usize;
        render_command_encoder
            .0
            .setCullMode(MTLCullMode(mode));
    }

    fn draw_primitives(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();
        let primitive_type = args.get_integer_arg(1).unwrap();
        let vertex_count = args.get_integer_arg(2).unwrap();
        let instance_count = args.get_integer_arg(3).unwrap();
        let vertex_start = args.get_integer_arg(4).unwrap();
        let base_instance = args.get_integer_arg(5).unwrap();

        unsafe {
            render_command_encoder
                .0
                .drawPrimitives_vertexStart_vertexCount_instanceCount_baseInstance(
                    MTLPrimitiveType(primitive_type as usize),
                    vertex_start as usize,
                    vertex_count as usize,
                    instance_count as usize,
                    base_instance as usize,
                );
        }
    }

    fn set_argument_table_object(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();

        let argument_table_instance = args.get_arg(1).unwrap();
        let argument_table = argument_table_instance.get_peer::<ArgumentTable>().unwrap();

        unsafe {
            render_command_encoder.0.setArgumentTable_atStages(
                argument_table.table.as_ref(),
                MTLRenderStages::Vertex | MTLRenderStages::Fragment,
            );
        }
    }

    fn intra_pass_barrier(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();
        let after_encoder_stages = args.get_integer_arg(1).unwrap() as usize;
        let before_encoder_stages = args.get_integer_arg(2).unwrap() as usize;
        let visibility_options = args.get_integer_arg(3).unwrap() as usize;
        render_command_encoder
            .0
            .barrierAfterEncoderStages_beforeEncoderStages_visibilityOptions(
                MTLStages(after_encoder_stages),
                MTLStages(before_encoder_stages),
                MTL4VisibilityOptions(visibility_options),
            );
    }

    fn consumer_barrier(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();
        let after_encoder_stages = args.get_integer_arg(1).unwrap() as usize;
        let before_encoder_stages = args.get_integer_arg(2).unwrap() as usize;
        let visibility_options = args.get_integer_arg(3).unwrap() as usize;
        render_command_encoder
            .0
            .barrierAfterQueueStages_beforeStages_visibilityOptions(
                MTLStages(after_encoder_stages),
                MTLStages(before_encoder_stages),
                MTL4VisibilityOptions(visibility_options),
            );
    }

    fn producer_barrier(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();
        let after_encoder_stages = args.get_integer_arg(1).unwrap() as usize;
        let before_encoder_stages = args.get_integer_arg(2).unwrap() as usize;
        let visibility_options = args.get_integer_arg(3).unwrap() as usize;

        render_command_encoder
            .0
            .barrierAfterStages_beforeQueueStages_visibilityOptions(
                MTLStages(after_encoder_stages),
                MTLStages(before_encoder_stages),
                MTL4VisibilityOptions(visibility_options),
            );
    }

    fn end_encoding(args: NativeArguments) {
        let render_command_encoder_instance = args.get_arg(0).unwrap();
        let render_command_encoder = render_command_encoder_instance
            .get_peer::<RenderCommandEncoder>()
            .unwrap();
        render_command_encoder.0.endEncoding();
    }
}
