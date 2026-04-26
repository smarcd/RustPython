use crate::PyObject;
use crate::pystate::with_vm;
use crate::handles::{exported_object_wrapper, resolve_object_handle};
use rustpython_vm::AsObject;
use rustpython_vm::builtins::PyStr;
use rustpython_vm::builtins::PyModule;

#[repr(C)]
pub struct PyModuleDef {
    _private: [u8; 0],
}

#[unsafe(no_mangle)]
pub extern "C" fn PyModuleDef_Init(def: *mut PyModuleDef) -> *mut PyObject {
    def.cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn PyModule_GetNameObject(module: *mut PyObject) -> *mut PyObject {
    with_vm(|vm| {
        let module = unsafe { &*resolve_object_handle(module) }.try_downcast_ref::<PyModule>(vm)?;
        module.get_attr("__name__", vm)
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn PyModule_NewObject(name: *mut PyObject) -> *mut PyObject {
    with_vm(|vm| {
        let name = unsafe { &*resolve_object_handle(name) }.try_downcast_ref::<PyStr>(vm)?;
        let name = name.to_string_lossy().into_owned();
        let module = vm.new_module(&name, vm.ctx.new_dict(), None);
        Ok(unsafe {
            exported_object_wrapper(module.as_object().as_raw().cast_mut(), core::mem::size_of::<usize>() * 2)
        })
    })
}
