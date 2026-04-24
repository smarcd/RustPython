use crate::handles::resolve_object_handle;
use crate::{PyObject, with_vm};
use core::ffi::c_double;

#[unsafe(no_mangle)]
pub extern "C" fn PyFloat_FromDouble(value: c_double) -> *mut PyObject {
    with_vm(|vm| vm.ctx.new_float(value))
}

#[unsafe(no_mangle)]
pub extern "C" fn PyFloat_AsDouble(obj: *mut PyObject) -> c_double {
    with_vm(|vm| {
        let obj = unsafe { &*resolve_object_handle(obj) };
        Ok(obj.try_float(vm)?.to_f64())
    })
}
