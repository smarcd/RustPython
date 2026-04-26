use crate::PyObject;
use core::ffi::c_long;
use rustpython_vm::AsObject;

#[unsafe(no_mangle)]
pub extern "C" fn PyBool_FromLong(value: c_long) -> *mut PyObject {
    crate::with_vm(|vm| {
        let obj = vm.ctx.new_bool(value != 0).as_object().to_owned();
        obj.into_raw().as_ptr()
    })
}
