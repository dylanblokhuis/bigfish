use bigfish_macros::native_func;

use crate::dart_api::NativeArguments;

#[native_func]
fn init_gpu(args: NativeArguments) {
    let a = args.get_boolean_arg(0);
    println!("Initializing GPU");
}
