use std::{ffi::CString, mem::MaybeUninit, path::Path};

use anyhow::Result;

mod bindings {
    #![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, unused)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub struct DartEngine {
    isolate: DartIsolate,
}

impl DartEngine {
    pub fn new(
        script_path: impl AsRef<Path>,
        package_config: impl AsRef<Path>,
        hmr: bool,
    ) -> Result<Self> {
        let script_path = script_path.as_ref().to_string_lossy().into_owned();
        let package_config = package_config.as_ref().to_string_lossy().into_owned();

        let config = bindings::DartDllConfig {
            service_port: 5858,
            start_service_isolate: hmr,
        };

        let isolate = unsafe {
            bindings::DartDll_Initialize(&config);
            let script_path = CString::new(script_path).unwrap();
            let package_config = CString::new(package_config).unwrap();
            let isolate = bindings::DartDll_LoadScript(
                script_path.as_ptr(),
                package_config.as_ptr(),
                std::ptr::null_mut(),
            );
            if isolate.is_null() {
                bindings::DartDll_Shutdown();
                return Err(anyhow::anyhow!("Failed to load script"));
            }
            isolate
        };

        Ok(Self {
            isolate: DartIsolate::new(isolate),
        })
    }

    pub fn isolate(&mut self) -> &mut DartIsolate {
        &mut self.isolate
    }
}

impl Drop for DartEngine {
    fn drop(&mut self) {
        unsafe {
            bindings::DartDll_Shutdown();
        }
    }
}

pub struct DartIsolate {
    inner: bindings::Dart_Isolate,
}

impl DartIsolate {
    pub fn new(inner: bindings::Dart_Isolate) -> Self {
        let mut this = Self { inner };
        this.enter().invoke("main", &mut []).unwrap();
        return this;
    }

    pub fn enter<'a>(&'a mut self) -> DartIsolateGuard<'a> {
        DartIsolateGuard::new(self)
    }
}

impl Drop for DartIsolate {
    fn drop(&mut self) {
        unsafe {
            bindings::Dart_ShutdownIsolate();
        }
    }
}

pub struct DartIsolateGuard<'a> {
    _isolate: &'a DartIsolate,
    library: bindings::Dart_Handle,
}

impl<'a> DartIsolateGuard<'a> {
    pub fn new(isolate: &'a DartIsolate) -> Self {
        let library = unsafe {
            bindings::Dart_EnterIsolate(isolate.inner);
            bindings::Dart_EnterScope();
            bindings::Dart_RootLibrary()
        };

        Self {
            _isolate: isolate,
            library,
        }
    }

    #[allow(unused)]
    pub fn exit(self) {
        drop(self);
    }

    pub fn invoke(
        &mut self,
        name: &str,
        args: &mut [bindings::Dart_Handle],
    ) -> Result<bindings::Dart_Handle> {
        let result = unsafe {
            bindings::Dart_Invoke(
                self.library,
                DartString::new(name).into(),
                args.len() as i32,
                args.as_mut_ptr() as *mut bindings::Dart_Handle,
            )
        };
        if unsafe { bindings::Dart_IsError(result) } {
            return Err(anyhow::anyhow!("Failed to invoke"));
        }
        Ok(result)
    }

    pub fn drain_microtask_queue(&mut self) {
        unsafe {
            bindings::DartDll_DrainMicrotaskQueue();
        }
    }
}

impl<'a> Drop for DartIsolateGuard<'a> {
    fn drop(&mut self) {
        unsafe {
            bindings::Dart_ExitScope();
            bindings::Dart_ExitIsolate();
        }
    }
}

pub struct DartString(bindings::Dart_Handle);

impl DartString {
    pub fn new(s: &str) -> Self {
        let s = CString::new(s).unwrap();
        Self(unsafe { bindings::Dart_NewStringFromCString(s.as_ptr()) })
    }

    pub fn from_handle(handle: DartHandle) -> Result<Self> {
        if !handle.is_string() {
            return Err(anyhow::anyhow!(
                "DartString::from_handle: handle is not a string"
            ));
        }

        Ok(Self(handle.0))
    }
}

impl Into<bindings::Dart_Handle> for DartString {
    fn into(self) -> bindings::Dart_Handle {
        self.0
    }
}

pub struct DartHandle(bindings::Dart_Handle);

pub enum DartObjectComparison {
    Equal,
    NotEqual,
    CannotCompare,
}

#[allow(unused)]
impl DartHandle {
    pub fn new(handle: bindings::Dart_Handle) -> Self {
        Self(handle)
    }

    pub fn identity_equals(&self, other: &Self) -> bool {
        unsafe { bindings::Dart_IdentityEquals(self.0, other.0) }
    }

    pub fn object_equals(&self, other: &Self) -> DartObjectComparison {
        let mut equal_result = MaybeUninit::<bool>::uninit();
        let is_success =
            unsafe { bindings::Dart_ObjectEquals(self.0, other.0, equal_result.as_mut_ptr()) };
        if is_success.is_null() {
            return DartObjectComparison::CannotCompare;
        }
        if unsafe { equal_result.assume_init() } {
            DartObjectComparison::Equal
        } else {
            DartObjectComparison::NotEqual
        }
    }

    pub fn object_is_type(&self, type_: &Self) -> bool {
        let mut instanceof = MaybeUninit::<bool>::uninit();
        let is_success =
            unsafe { bindings::Dart_ObjectIsType(self.0, type_.0, instanceof.as_mut_ptr()) };
        if is_success.is_null() {
            return false;
        }
        unsafe { instanceof.assume_init() }
    }

    pub fn is_instance(&self) -> bool {
        unsafe { bindings::Dart_IsInstance(self.0) }
    }

    pub fn is_number(&self) -> bool {
        unsafe { bindings::Dart_IsNumber(self.0) }
    }

    pub fn is_integer(&self) -> bool {
        unsafe { bindings::Dart_IsInteger(self.0) }
    }

    pub fn is_double(&self) -> bool {
        unsafe { bindings::Dart_IsDouble(self.0) }
    }

    pub fn is_boolean(&self) -> bool {
        unsafe { bindings::Dart_IsBoolean(self.0) }
    }

    pub fn is_string(&self) -> bool {
        unsafe { bindings::Dart_IsString(self.0) }
    }

    pub fn is_string_latin1(&self) -> bool {
        unsafe { bindings::Dart_IsStringLatin1(self.0) }
    }

    pub fn is_list(&self) -> bool {
        unsafe { bindings::Dart_IsList(self.0) }
    }

    pub fn is_map(&self) -> bool {
        unsafe { bindings::Dart_IsMap(self.0) }
    }

    pub fn is_library(&self) -> bool {
        unsafe { bindings::Dart_IsLibrary(self.0) }
    }

    pub fn is_type(&self) -> bool {
        unsafe { bindings::Dart_IsType(self.0) }
    }

    pub fn is_function(&self) -> bool {
        unsafe { bindings::Dart_IsFunction(self.0) }
    }

    pub fn is_variable(&self) -> bool {
        unsafe { bindings::Dart_IsVariable(self.0) }
    }

    pub fn is_type_variable(&self) -> bool {
        unsafe { bindings::Dart_IsTypeVariable(self.0) }
    }

    pub fn is_closure(&self) -> bool {
        unsafe { bindings::Dart_IsClosure(self.0) }
    }

    pub fn is_typed_data(&self) -> bool {
        unsafe { bindings::Dart_IsTypedData(self.0) }
    }

    pub fn is_byte_buffer(&self) -> bool {
        unsafe { bindings::Dart_IsByteBuffer(self.0) }
    }

    pub fn is_future(&self) -> bool {
        unsafe { bindings::Dart_IsFuture(self.0) }
    }

    pub fn instance_get_type(&self) -> Result<DartHandle> {
        let a = unsafe { bindings::Dart_InstanceGetType(self.0) };
        if a.is_null() {
            return Err(anyhow::anyhow!("Failed to get instance type"));
        }
        Ok(DartHandle::new(a))
    }

    pub fn class_name(&self) -> Result<DartString> {
        let a = unsafe { bindings::Dart_ClassName(self.0) };
        DartString::from_handle(DartHandle::new(a))
    }

    pub fn function_name(&self) -> Result<DartString> {
        let a = unsafe { bindings::Dart_FunctionName(self.0) };
        DartString::from_handle(DartHandle::new(a))
    }
}
