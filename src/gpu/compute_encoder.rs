use bigfish_macros::native_impl;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_metal::{MTL4ComputeCommandEncoder, MTL4VisibilityOptions, MTLSize, MTLStages};

use crate::dart_api::{NativeArguments, Scope};

use super::resources::{ArgumentTable, ComputePipeline, Texture};

type Id<T> = Retained<ProtocolObject<T>>;

pub struct ComputeCommandEncoder(pub Id<dyn MTL4ComputeCommandEncoder>);

#[native_impl]
impl ComputeCommandEncoder {
    fn set_compute_pipeline(args: NativeArguments) {
        let compute_command_encoder_instance = args.get_arg(0).unwrap();
        let compute_command_encoder = compute_command_encoder_instance
            .get_peer::<ComputeCommandEncoder>()
            .unwrap();
        let compute_pipeline = args.get_arg(1).unwrap();
        let compute_pipeline = compute_pipeline.get_peer::<ComputePipeline>().unwrap();
        compute_command_encoder
            .0
            .setComputePipelineState(&compute_pipeline.compute_pipeline_state);
    }

    fn set_argument_table_object(args: NativeArguments) {
        let compute_command_encoder_instance = args.get_arg(0).unwrap();
        let compute_command_encoder = compute_command_encoder_instance
            .get_peer::<ComputeCommandEncoder>()
            .unwrap();
        let argument_table_instance = args.get_arg(1).unwrap();
        let argument_table = argument_table_instance.get_peer::<ArgumentTable>().unwrap();
        compute_command_encoder
            .0
            .setArgumentTable(Some(argument_table.table.as_ref()));
    }

    fn dispatch_threads(args: NativeArguments) {
        let compute_command_encoder_instance = args.get_arg(0).unwrap();
        let compute_command_encoder = compute_command_encoder_instance
            .get_peer::<ComputeCommandEncoder>()
            .unwrap();
        let threads_per_grid_x = args.get_integer_arg(1).unwrap() as usize;
        let threads_per_grid_y = args.get_integer_arg(2).unwrap() as usize;
        let threads_per_grid_z = args.get_integer_arg(3).unwrap() as usize;
        let threads_per_threadgroup_x = args.get_integer_arg(4).unwrap() as usize;
        let threads_per_threadgroup_y = args.get_integer_arg(5).unwrap() as usize;
        let threads_per_threadgroup_z = args.get_integer_arg(6).unwrap() as usize;

        compute_command_encoder
            .0
            .dispatchThreads_threadsPerThreadgroup(
                MTLSize {
                    width: threads_per_grid_x,
                    height: threads_per_grid_y,
                    depth: threads_per_grid_z,
                },
                MTLSize {
                    width: threads_per_threadgroup_x,
                    height: threads_per_threadgroup_y,
                    depth: threads_per_threadgroup_z,
                },
            );
    }

    fn dispatch_threadgroups(args: NativeArguments) {
        let compute_command_encoder_instance = args.get_arg(0).unwrap();
        let compute_command_encoder = compute_command_encoder_instance
            .get_peer::<ComputeCommandEncoder>()
            .unwrap();
        let threadgroups_per_grid_x = args.get_integer_arg(1).unwrap() as usize;
        let threadgroups_per_grid_y = args.get_integer_arg(2).unwrap() as usize;
        let threadgroups_per_grid_z = args.get_integer_arg(3).unwrap() as usize;
        let threads_per_threadgroup_x = args.get_integer_arg(4).unwrap() as usize;
        let threads_per_threadgroup_y = args.get_integer_arg(5).unwrap() as usize;
        let threads_per_threadgroup_z = args.get_integer_arg(6).unwrap() as usize;

        compute_command_encoder
            .0
            .dispatchThreadgroups_threadsPerThreadgroup(
                MTLSize {
                    width: threadgroups_per_grid_x,
                    height: threadgroups_per_grid_y,
                    depth: threadgroups_per_grid_z,
                },
                MTLSize {
                    width: threads_per_threadgroup_x,
                    height: threads_per_threadgroup_y,
                    depth: threads_per_threadgroup_z,
                },
            );
    }

    fn intra_pass_barrier(args: NativeArguments) {
        let compute_command_encoder_instance = args.get_arg(0).unwrap();
        let compute_command_encoder = compute_command_encoder_instance
            .get_peer::<ComputeCommandEncoder>()
            .unwrap();
        let after_encoder_stages = args.get_integer_arg(1).unwrap() as usize;
        let before_encoder_stages = args.get_integer_arg(2).unwrap() as usize;
        let visibility_options = args.get_integer_arg(3).unwrap() as usize;
        compute_command_encoder
            .0
            .barrierAfterEncoderStages_beforeEncoderStages_visibilityOptions(
                MTLStages(after_encoder_stages),
                MTLStages(before_encoder_stages),
                MTL4VisibilityOptions(visibility_options),
            );
    }

    fn consumer_barrier(args: NativeArguments) {
        let compute_command_encoder_instance = args.get_arg(0).unwrap();
        let compute_command_encoder = compute_command_encoder_instance
            .get_peer::<ComputeCommandEncoder>()
            .unwrap();
        let after_encoder_stages = args.get_integer_arg(1).unwrap() as usize;
        let before_encoder_stages = args.get_integer_arg(2).unwrap() as usize;
        let visibility_options = args.get_integer_arg(3).unwrap() as usize;
        compute_command_encoder
            .0
            .barrierAfterQueueStages_beforeStages_visibilityOptions(
                MTLStages(after_encoder_stages),
                MTLStages(before_encoder_stages),
                MTL4VisibilityOptions(visibility_options),
            );
    }

    fn producer_barrier(args: NativeArguments) {
        let compute_command_encoder_instance = args.get_arg(0).unwrap();
        let compute_command_encoder = compute_command_encoder_instance
            .get_peer::<ComputeCommandEncoder>()
            .unwrap();
        let after_encoder_stages = args.get_integer_arg(1).unwrap() as usize;
        let before_encoder_stages = args.get_integer_arg(2).unwrap() as usize;
        let visibility_options = args.get_integer_arg(3).unwrap() as usize;

        compute_command_encoder
            .0
            .barrierAfterStages_beforeQueueStages_visibilityOptions(
                MTLStages(after_encoder_stages),
                MTLStages(before_encoder_stages),
                MTL4VisibilityOptions(visibility_options),
            );
    }

    fn copy(args: NativeArguments) {
        let compute_command_encoder_instance = args.get_arg(0).unwrap();
        let compute_command_encoder = compute_command_encoder_instance
            .get_peer::<ComputeCommandEncoder>()
            .unwrap();
        let source_texture_instance = args.get_arg(1).unwrap();
        let source_texture = source_texture_instance.get_peer::<Texture>().unwrap();
        let destination_texture_instance = args.get_arg(2).unwrap();
        let destination_texture = destination_texture_instance.get_peer::<Texture>().unwrap();
        unsafe {
            compute_command_encoder.0.copyFromTexture_toTexture(
                source_texture.texture.as_ref(),
                destination_texture.texture.as_ref(),
            );
        }
    }

    fn generate_mipmaps(args: NativeArguments) {
        let compute_command_encoder_instance = args.get_arg(0).unwrap();
        let compute_command_encoder = compute_command_encoder_instance
            .get_peer::<ComputeCommandEncoder>()
            .unwrap();
        let texture_instance = args.get_arg(1).unwrap();
        let texture = texture_instance.get_peer::<Texture>().unwrap();
        unsafe {
            // compute_command_encoder.0.barrier
            compute_command_encoder
                .0
                .generateMipmapsForTexture(texture.texture.as_ref());
        }
    }

    fn end_encoding(args: NativeArguments) {
        let compute_command_encoder_instance = args.get_arg(0).unwrap();
        let compute_command_encoder = compute_command_encoder_instance
            .get_peer::<ComputeCommandEncoder>()
            .unwrap();
        compute_command_encoder.0.endEncoding();
    }
}
