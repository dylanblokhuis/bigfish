use std::path::Path;

use anyhow::Result;

use crate::dart_api::{Isolate, Runtime, Scope};
use crate::sys;

pub struct DartEngine {
    _runtime: Runtime,
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

        let config = sys::DartDllConfig {
            service_port: 5858,
            start_service_isolate: hmr,
        };

        let runtime = Runtime::initialize(&config).map_err(|e| anyhow::anyhow!(e))?;

        let script_path =
            std::ffi::CString::new(script_path).map_err(|_| anyhow::anyhow!("bad script path"))?;
        let package_config = std::ffi::CString::new(package_config)
            .map_err(|_| anyhow::anyhow!("bad package_config path"))?;

        let isolate = runtime
            .load_script(&script_path, &package_config, std::ptr::null_mut())
            .map_err(|e| anyhow::anyhow!(e))?;

        Ok(Self {
            _runtime: runtime,
            isolate: DartIsolate::new(isolate)?,
        })
    }

    pub fn isolate(&mut self) -> &mut DartIsolate {
        &mut self.isolate
    }
}

pub struct DartIsolate {
    inner: Isolate,
}

impl DartIsolate {
    pub fn new(inner: Isolate) -> Result<Self> {
        let mut this = Self { inner };
        {
            let mut scope = this.enter();
            scope.invoke("main", &mut []).map_err(|e| anyhow::anyhow!(e))?;
        }
        Ok(this)
    }

    pub fn enter<'a>(&'a mut self) -> DartIsolateGuard<'a> {
        DartIsolateGuard {
            scope: self.inner.enter(),
        }
    }
}

pub struct DartIsolateGuard<'a> {
    scope: Scope<'a>,
}

impl<'a> DartIsolateGuard<'a> {
    #[allow(unused)]
    pub fn exit(self) {
        drop(self);
    }

    pub fn invoke(
        &mut self,
        name: &str,
        args: &mut [sys::Dart_Handle],
    ) -> Result<sys::Dart_Handle> {
        Ok(self
            .scope
            .invoke(name, args)
            .map_err(|e| anyhow::anyhow!(e))?
            .raw())
    }

    pub fn drain_microtask_queue(&mut self) {
        // Best-effort: drain returns a handle (may be error), but this sample ignores it.
        let _ = self.scope.check(unsafe { sys::DartDll_DrainMicrotaskQueue() });
    }
}

// The higher-level handle utilities moved to `crate::dart_api`.
