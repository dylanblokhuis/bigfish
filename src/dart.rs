#![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, unused)]

use std::{ffi::CString, ops::Deref, path::Path, str::FromStr};

use anyhow::Result;

mod bindings {
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
            let config = bindings::DartDll_Initialize(&config);
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
    isolate: &'a DartIsolate,
    library: bindings::Dart_Handle,
}

impl<'a> DartIsolateGuard<'a> {
    pub fn new(isolate: &'a DartIsolate) -> Self {
        let library = unsafe {
            bindings::Dart_EnterIsolate(isolate.inner);
            bindings::Dart_EnterScope();
            bindings::Dart_RootLibrary()
        };

        Self { isolate, library }
    }

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
}

impl Into<bindings::Dart_Handle> for DartString {
    fn into(self) -> bindings::Dart_Handle {
        self.0
    }
}
