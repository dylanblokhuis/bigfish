pub mod sys {
    #![allow(
        non_upper_case_globals,
        non_camel_case_types,
        non_snake_case,
        unused,
        clippy::all
    )]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
    mem::{ManuallyDrop, MaybeUninit},
    os::raw::c_void,
    ptr,
};

pub type Result<T> = std::result::Result<T, DartError>;

#[derive(Debug, thiserror::Error)]
pub enum DartError {
    #[error("dart api returned null handle")]
    NullHandle,
    #[error("dart api error: {0}")]
    Api(String),
}

impl DartError {
    fn from_error_handle(handle: sys::Dart_Handle) -> Self {
        unsafe {
            let msg_ptr = sys::Dart_GetError(handle);
            if msg_ptr.is_null() {
                return DartError::Api("<Dart_GetError returned null>".to_string());
            }
            let msg = CStr::from_ptr(msg_ptr).to_string_lossy().into_owned();
            DartError::Api(msg)
        }
    }
}

fn check(handle: sys::Dart_Handle) -> Result<sys::Dart_Handle> {
    if handle.is_null() {
        return Err(DartError::NullHandle);
    }
    if unsafe { sys::Dart_IsError(handle) } {
        return Err(DartError::from_error_handle(handle));
    }
    Ok(handle)
}

/// A running Dart VM instance (initialized via `DartDll_Initialize`).
///
/// Dropping this will call `DartDll_Shutdown()`.
pub struct Runtime {
    _priv: (),
}

pub struct RuntimeConfig {
    pub service_port: u16,
    pub start_service_isolate: bool,
}

impl RuntimeConfig {
    pub fn new(service_port: u16, start_service_isolate: bool) -> Self {
        Self {
            service_port,
            start_service_isolate,
        }
    }
}
impl Runtime {
    pub fn initialize(config: RuntimeConfig) -> Result<Self> {
        let config = sys::DartDllConfig {
            service_port: config.service_port as i32,
            start_service_isolate: config.start_service_isolate,
        };
        let ok = unsafe { sys::DartDll_Initialize(&config) };
        if !ok {
            return Err(DartError::Api("DartDll_Initialize returned false".into()));
        }
        Ok(Self { _priv: () })
    }

    pub fn load_script(
        &self,
        script_uri: &CStr,
        package_config: &CStr,
        isolate_data: *mut c_void,
    ) -> Result<Isolate> {
        let isolate = unsafe {
            sys::DartDll_LoadScript(script_uri.as_ptr(), package_config.as_ptr(), isolate_data)
        };
        if isolate.is_null() {
            return Err(DartError::Api(
                "DartDll_LoadScript returned null isolate".into(),
            ));
        }
        let mut isolate = Isolate { raw: isolate };
        {
            let scope = isolate.enter();
            let library = scope.library();
            unsafe { sys::Dart_SetNativeResolver(library.raw(), Some(native_resolver), None) };
        }

        Ok(isolate)
    }

    pub fn drain_microtask_queue<'i>(&self, scope: &Scope<'i>) -> Result<Handle<'i>> {
        scope.check(unsafe { sys::DartDll_DrainMicrotaskQueue() })
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        unsafe {
            let _ = sys::DartDll_Shutdown();
        }
    }
}

/// A Dart isolate created/loaded through the embedding API.
pub struct Isolate {
    raw: sys::Dart_Isolate,
}

impl Isolate {
    /// Creates a scope that Rust won't exit, it will be handled by the Dart VM. Hence we use ManuallyDrop to avoid double drop.
    pub fn current<'i>() -> Result<ManuallyDrop<Scope<'i>>> {
        let isolate = unsafe { sys::Dart_CurrentIsolate() };
        if isolate.is_null() {
            return Err(DartError::Api("Dart_CurrentIsolate returned null".into()));
        }
        Ok(ManuallyDrop::new(Scope {
            library: unsafe { sys::Dart_RootLibrary() },
            _marker: PhantomData,
        }))
    }

    pub fn enter(&mut self) -> Scope<'_> {
        unsafe {
            sys::Dart_EnterIsolate(self.raw);
            sys::Dart_EnterScope();
            let lib = sys::Dart_RootLibrary();
            Scope {
                library: lib,
                _marker: PhantomData,
            }
        }
    }
}

impl Drop for Isolate {
    fn drop(&mut self) {
        // Best-effort isolate shutdown. The API requires a current isolate.
        unsafe {
            if !self.raw.is_null() {
                sys::Dart_EnterIsolate(self.raw);
                sys::Dart_ShutdownIsolate();
                sys::Dart_ExitIsolate();
            }
        }
    }
}

/// A Dart API scope bound to an entered isolate.
///
/// All [`Handle`] values produced from this scope are only valid until this is dropped.
pub struct Scope<'i> {
    library: sys::Dart_Handle,
    _marker: PhantomData<&'i mut ()>,
}

impl<'i> Scope<'i> {
    pub fn library(&self) -> Handle<'i> {
        Handle {
            raw: self.library,
            _marker: PhantomData,
        }
    }

    pub fn check(&self, handle: sys::Dart_Handle) -> Result<Handle<'i>> {
        let handle = check(handle)?;
        Ok(Handle {
            raw: handle,
            _marker: PhantomData,
        })
    }

    pub fn new_string(&self, s: &str) -> Result<Handle<'i>> {
        let s =
            CString::new(s).map_err(|_| DartError::Api("string contained interior NUL".into()))?;
        self.check(unsafe { sys::Dart_NewStringFromCString(s.as_ptr()) })
    }

    pub fn invoke(&mut self, name: &str, args: &mut [sys::Dart_Handle]) -> Result<Handle<'i>> {
        let name = self.new_string(name)?;
        self.check(unsafe {
            sys::Dart_Invoke(self.library, name.raw, args.len() as i32, args.as_mut_ptr())
        })
    }

    pub fn instance_get_type(&self, instance: Handle<'i>) -> Result<Handle<'i>> {
        self.check(unsafe { sys::Dart_InstanceGetType(instance.raw) })
    }

    pub fn class_name(&self, cls_type: Handle<'i>) -> Result<Handle<'i>> {
        self.check(unsafe { sys::Dart_ClassName(cls_type.raw) })
    }

    pub fn function_name(&self, function: Handle<'i>) -> Result<Handle<'i>> {
        self.check(unsafe { sys::Dart_FunctionName(function.raw) })
    }

    pub fn get_error_message(&mut self, error: sys::Dart_Handle) -> String {
        unsafe {
            let msg_ptr = sys::Dart_GetError(error);
            if msg_ptr.is_null() {
                "<Dart_GetError returned null>".to_string()
            } else {
                CStr::from_ptr(msg_ptr).to_string_lossy().into_owned()
            }
        }
    }

    pub fn get_class(&self, class_name: &str) -> Result<Handle<'i>> {
        let class_name = self.new_string(class_name)?;
        self.check(unsafe { sys::Dart_GetClass(self.library, class_name.raw) })
    }
}

impl Drop for Scope<'_> {
    fn drop(&mut self) {
        unsafe {
            sys::Dart_ExitScope();
            sys::Dart_ExitIsolate();
        }
    }
}

/// A non-owning handle that is only valid for the lifetime of its [`Scope`].
#[repr(transparent)]
pub struct Handle<'s> {
    raw: sys::Dart_Handle,
    _marker: PhantomData<&'s ()>,
}

impl<'s> Copy for Handle<'s> {}
impl<'s> Clone for Handle<'s> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'s> Handle<'s> {
    pub fn raw(self) -> sys::Dart_Handle {
        self.raw
    }

    pub fn is_string(self) -> bool {
        unsafe { sys::Dart_IsString(self.raw) }
    }

    pub fn is_integer(self) -> bool {
        unsafe { sys::Dart_IsInteger(self.raw) }
    }

    pub fn is_double(self) -> bool {
        unsafe { sys::Dart_IsDouble(self.raw) }
    }

    pub fn is_boolean(self) -> bool {
        unsafe { sys::Dart_IsBoolean(self.raw) }
    }

    pub fn is_list(self) -> bool {
        unsafe { sys::Dart_IsList(self.raw) }
    }

    pub fn is_typed_data(self) -> bool {
        unsafe { sys::Dart_IsTypedData(self.raw) }
    }

    pub fn identity_equals(self, other: Handle<'s>) -> bool {
        unsafe { sys::Dart_IdentityEquals(self.raw, other.raw) }
    }

    pub fn object_equals(self, other: Handle<'s>) -> Result<bool> {
        let mut out = MaybeUninit::<bool>::uninit();
        check(unsafe { sys::Dart_ObjectEquals(self.raw, other.raw, out.as_mut_ptr()) })?;
        Ok(unsafe { out.assume_init() })
    }

    pub fn object_is_type(self, type_obj: Handle<'s>) -> Result<bool> {
        let mut out = MaybeUninit::<bool>::uninit();
        check(unsafe { sys::Dart_ObjectIsType(self.raw, type_obj.raw, out.as_mut_ptr()) })?;
        Ok(unsafe { out.assume_init() })
    }

    pub fn to_bool(self) -> Result<bool> {
        let mut out = MaybeUninit::<bool>::uninit();
        check(unsafe { sys::Dart_BooleanValue(self.raw, out.as_mut_ptr()) })?;
        Ok(unsafe { out.assume_init() })
    }

    pub fn to_i64(self) -> Result<i64> {
        let mut out = MaybeUninit::<i64>::uninit();
        check(unsafe { sys::Dart_IntegerToInt64(self.raw, out.as_mut_ptr()) })?;
        Ok(unsafe { out.assume_init() })
    }

    pub fn to_u64(self) -> Result<u64> {
        let mut out = MaybeUninit::<u64>::uninit();
        check(unsafe { sys::Dart_IntegerToUint64(self.raw, out.as_mut_ptr()) })?;
        Ok(unsafe { out.assume_init() })
    }

    pub fn to_f64(self) -> Result<f64> {
        let mut out = MaybeUninit::<f64>::uninit();
        check(unsafe { sys::Dart_DoubleValue(self.raw, out.as_mut_ptr()) })?;
        Ok(unsafe { out.assume_init() })
    }

    pub fn to_utf8(self) -> Result<Vec<u8>> {
        let mut ptr_out = MaybeUninit::<*mut u8>::uninit();
        let mut len_out = MaybeUninit::<isize>::uninit();
        check(unsafe {
            sys::Dart_StringToUTF8(
                self.raw,
                ptr_out.as_mut_ptr(),
                len_out.as_mut_ptr() as *mut isize,
            )
        })?;

        let ptr = unsafe { ptr_out.assume_init() };
        let len = unsafe { len_out.assume_init() };
        if ptr.is_null() || len < 0 {
            return Err(DartError::Api(
                "Dart_StringToUTF8 returned null/negative".into(),
            ));
        }
        let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
        Ok(slice.to_vec())
    }

    pub fn to_string_lossy(self) -> Result<String> {
        let bytes = self.to_utf8()?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }

    pub fn is_null(self) -> bool {
        unsafe { sys::Dart_IsNull(self.raw) }
    }
}

/// Safe wrapper for Dart_NativeArguments
pub struct NativeArguments<'a> {
    raw: sys::Dart_NativeArguments,
    _marker: PhantomData<&'a ()>,
}

impl<'a> NativeArguments<'a> {
    pub fn from_raw(raw: sys::Dart_NativeArguments) -> Self {
        Self {
            raw,
            _marker: PhantomData,
        }
    }

    pub fn raw(&self) -> sys::Dart_NativeArguments {
        self.raw
    }

    pub fn get_arg_count(&self) -> i32 {
        unsafe { sys::Dart_GetNativeArgumentCount(self.raw) }
    }

    pub fn get_arg(&self, index: i32) -> Result<Handle<'a>> {
        let handle = unsafe { sys::Dart_GetNativeArgument(self.raw, index) };
        if handle.is_null() {
            return Err(DartError::NullHandle);
        }
        if unsafe { sys::Dart_IsError(handle) } {
            return Err(DartError::from_error_handle(handle));
        }
        Ok(Handle {
            raw: handle,
            _marker: PhantomData,
        })
    }

    pub fn get_string_arg(&self, index: i32) -> Result<Handle<'a>> {
        let mut peer: *mut c_void = ptr::null_mut();
        let handle = unsafe { sys::Dart_GetNativeStringArgument(self.raw, index, &mut peer) };
        if handle.is_null() {
            return Err(DartError::NullHandle);
        }
        if unsafe { sys::Dart_IsError(handle) } {
            return Err(DartError::from_error_handle(handle));
        }
        Ok(Handle {
            raw: handle,
            _marker: PhantomData,
        })
    }

    pub fn get_integer_arg(&self, index: i32) -> Result<i64> {
        let mut val: i64 = 0;
        let handle = unsafe { sys::Dart_GetNativeIntegerArgument(self.raw, index, &mut val) };
        check(handle)?;
        Ok(val)
    }

    pub fn get_boolean_arg(&self, index: i32) -> Result<bool> {
        let mut val: bool = false;
        let handle = unsafe { sys::Dart_GetNativeBooleanArgument(self.raw, index, &mut val) };
        check(handle)?;
        Ok(val)
    }

    pub fn get_double_arg(&self, index: i32) -> Result<f64> {
        let mut val: f64 = 0.0;
        let handle = unsafe { sys::Dart_GetNativeDoubleArgument(self.raw, index, &mut val) };
        check(handle)?;
        Ok(val)
    }

    pub fn get_native_receiver(&self) -> Result<isize> {
        let mut val: isize = 0;
        let handle = unsafe { sys::Dart_GetNativeReceiver(self.raw, &mut val) };
        check(handle)?;
        Ok(val)
    }

    pub fn get_native_fields_of_arg(&self, index: i32, fields: &mut [isize]) -> Result<()> {
        let handle = unsafe {
            sys::Dart_GetNativeFieldsOfArgument(
                self.raw,
                index,
                fields.len() as i32,
                fields.as_mut_ptr(),
            )
        };
        check(handle)?;
        Ok(())
    }

    pub fn set_return_value(&self, handle: Handle<'a>) {
        unsafe { sys::Dart_SetReturnValue(self.raw, handle.raw()) }
    }

    pub fn set_bool_return_value(&self, val: bool) {
        unsafe { sys::Dart_SetBooleanReturnValue(self.raw, val) }
    }

    pub fn set_int_return_value(&self, val: i64) {
        unsafe { sys::Dart_SetIntegerReturnValue(self.raw, val) }
    }

    pub fn set_double_return_value(&self, val: f64) {
        unsafe { sys::Dart_SetDoubleReturnValue(self.raw, val) }
    }
}

pub struct List<'s>(Handle<'s>);

impl<'s> List<'s> {
    pub fn new(handle: Handle<'s>) -> Result<Self> {
        if !handle.is_list() {
            return Err(DartError::Api("expected a Dart List".into()));
        }
        Ok(Self(handle))
    }

    pub fn len(&self) -> Result<isize> {
        let mut out = MaybeUninit::<isize>::uninit();
        check(unsafe { sys::Dart_ListLength(self.0.raw, out.as_mut_ptr() as *mut isize) })?;
        Ok(unsafe { out.assume_init() })
    }

    pub fn get(&self, scope: &Scope<'s>, index: isize) -> Result<Handle<'s>> {
        scope.check(unsafe { sys::Dart_ListGetAt(self.0.raw, index) })
    }

    pub fn set(&self, index: isize, value: Handle<'s>) -> Result<()> {
        check(unsafe { sys::Dart_ListSetAt(self.0.raw, index, value.raw) })?;
        Ok(())
    }
}

/// A borrowed view over a Dart `TypedData` / `ByteData` buffer, released on drop.
pub struct TypedDataView<'s> {
    object: Handle<'s>,
    pub ty: sys::Dart_TypedData_Type,
    pub data: *mut u8,
    pub len: isize,
}

impl<'s> TypedDataView<'s> {
    pub fn acquire(object: Handle<'s>) -> Result<Self> {
        let mut ty = MaybeUninit::<sys::Dart_TypedData_Type>::uninit();
        let mut data = MaybeUninit::<*mut c_void>::uninit();
        let mut len = MaybeUninit::<isize>::uninit();

        check(unsafe {
            sys::Dart_TypedDataAcquireData(
                object.raw,
                ty.as_mut_ptr(),
                data.as_mut_ptr(),
                len.as_mut_ptr(),
            )
        })?;

        let data = unsafe { data.assume_init() } as *mut u8;
        let len = unsafe { len.assume_init() };
        Ok(Self {
            object,
            ty: unsafe { ty.assume_init() },
            data,
            len,
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        if self.data.is_null() || self.len <= 0 {
            return &[];
        }
        unsafe { std::slice::from_raw_parts(self.data, self.len as usize) }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        if self.data.is_null() || self.len <= 0 {
            return &mut [];
        }
        unsafe { std::slice::from_raw_parts_mut(self.data, self.len as usize) }
    }
}

impl Drop for TypedDataView<'_> {
    fn drop(&mut self) {
        unsafe {
            let _ = sys::Dart_TypedDataReleaseData(self.object.raw);
        }
    }
}

/// Convenience: a "null" Dart handle.
pub fn null_handle<'s>(scope: &Scope<'s>) -> Handle<'s> {
    // Dart_Null() should never be an error.
    scope
        .check(unsafe { sys::Dart_Null() })
        .unwrap_or_else(|_| Handle {
            raw: ptr::null_mut(),
            _marker: PhantomData,
        })
}

unsafe extern "C" fn native_resolver(
    name: sys::Dart_Handle,
    _num_of_arguments: ::std::os::raw::c_int,
    _auto_setup_scope: *mut bool,
) -> sys::Dart_NativeFunction {
    let mut cstr = MaybeUninit::<*const i8>::uninit();
    let res = sys::Dart_StringToCString(name, cstr.as_mut_ptr());
    debug_assert!(!res.is_null(), "Dart_StringToCString returned null");
    let name = CStr::from_ptr(cstr.assume_init());
    for function in inventory::iter::<NativeFunction>() {
        if function.name == name.to_str().unwrap() {
            return Some(function.function);
        }
    }

    None
}

pub struct NativeFunction {
    name: &'static str,
    function: unsafe extern "C" fn(args: sys::Dart_NativeArguments),
}

impl NativeFunction {
    pub const fn new(
        name: &'static str,
        function: unsafe extern "C" fn(args: sys::Dart_NativeArguments),
    ) -> Self {
        Self { name, function }
    }
}

inventory::collect!(NativeFunction);
