use crate::{PyObject, with_vm};
use crate::handles::{exported_object_wrapper, resolve_object_handle};
use core::ffi::{CStr, c_char};
use rustpython_vm::AsObject;
use rustpython_vm::builtins::{PyStr, PyStrRef, PyTuple};

#[unsafe(no_mangle)]
pub extern "C" fn PyImport_Import(name: *mut PyObject) -> *mut PyObject {
    with_vm(|vm| {
        let name = unsafe { (&*resolve_object_handle(name)).try_downcast_ref::<PyStr>(vm)? };
        let imported = if name.to_string_lossy().contains('.') {
            let from_list = PyTuple::<PyStrRef>::new_ref_typed(vec![vm.ctx.new_str("*")], &vm.ctx);
            vm.import_from(name, &from_list, 0)?
        } else {
            vm.import(name, 0)?
        };
        let raw = imported.as_object().as_raw().cast_mut();
        Ok(unsafe { exported_object_wrapper(raw, core::mem::size_of::<usize>() * 2) })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn PyImport_AddModuleRef(name: *const c_char) -> *mut PyObject {
    with_vm(|vm| {
        let name = unsafe { CStr::from_ptr(name) }
            .to_str()
            .expect("Name is not valid UTF-8");

        // TODO check if module already exists and return it if so, instead of creating a new one

        let module = vm.new_module(name, vm.ctx.new_dict(), None);
        Ok(unsafe {
            exported_object_wrapper(module.as_object().as_raw().cast_mut(), core::mem::size_of::<usize>() * 2)
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn PyImport_AddModule(name: *const c_char) -> *mut PyObject {
    PyImport_AddModuleRef(name)
}

#[cfg(test)]
mod tests {
    use pyo3::prelude::*;

    #[test]
    fn test_import() {
        Python::attach(|py| {
            let _module = py.import("sys").unwrap();
        })
    }
}
