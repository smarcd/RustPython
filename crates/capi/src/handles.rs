use crate::PyObject;
use crate::object::PyTypeObject;
use core::ffi::{c_char, c_ulong, c_void};
use core::ptr;
use rustpython_vm::{AsObject, Py, PyObjectRef};
use rustpython_vm::builtins::PyType;
use rustpython_vm::vm::Context;
use rustpython_vm::vm::thread::try_with_current_vm;
use std::alloc::{Layout, alloc_zeroed};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[repr(C)]
struct CApiObjectHeader {
    ob_refcnt: isize,
    ob_type: *mut PyTypeObject,
}

#[repr(C)]
struct ExportedStaticObject {
    ob_refcnt: isize,
    ob_type: *mut PyTypeObject,
}

#[repr(C)]
struct CApiVarObjectHeader {
    ob_base: CApiObjectHeader,
    ob_size: isize,
}

#[repr(C)]
struct CApiTypeObjectPrefix {
    ob_base: CApiVarObjectHeader,
    tp_name: *const c_char,
    tp_basicsize: isize,
    tp_itemsize: isize,
    tp_dealloc: *mut c_void,
    tp_vectorcall_offset: isize,
    tp_getattr: *mut c_void,
    tp_setattr: *mut c_void,
    tp_as_async: *mut c_void,
    tp_repr: *mut c_void,
    tp_as_number: *mut c_void,
    tp_as_sequence: *mut c_void,
    tp_as_mapping: *mut c_void,
    tp_hash: *mut c_void,
    tp_call: *mut c_void,
    tp_str: *mut c_void,
    tp_getattro: *mut c_void,
    tp_setattro: *mut c_void,
    tp_as_buffer: *mut c_void,
    tp_flags: c_ulong,
}

#[derive(Default)]
struct WrapperMaps {
    inner_to_wrapper: HashMap<usize, usize>,
    wrapper_to_inner: HashMap<usize, usize>,
}

fn wrapper_maps() -> &'static Mutex<WrapperMaps> {
    static WRAPPER_MAPS: OnceLock<Mutex<WrapperMaps>> = OnceLock::new();
    WRAPPER_MAPS.get_or_init(|| Mutex::new(WrapperMaps::default()))
}

fn retained_builtin_objects() -> &'static Mutex<Vec<PyObjectRef>> {
    static RETAINED_BUILTINS: OnceLock<Mutex<Vec<PyObjectRef>>> = OnceLock::new();
    RETAINED_BUILTINS.get_or_init(|| Mutex::new(Vec::new()))
}

#[inline]
fn normalize_type_ptr(ptr: *mut PyTypeObject) -> *mut PyTypeObject {
    ptr.map_addr(|addr| addr & !1)
}

const PY_TPFLAGS_LONG_SUBCLASS: c_ulong = 1 << 24;
const PY_TPFLAGS_LIST_SUBCLASS: c_ulong = 1 << 25;
const PY_TPFLAGS_TUPLE_SUBCLASS: c_ulong = 1 << 26;
const PY_TPFLAGS_BYTES_SUBCLASS: c_ulong = 1 << 27;
const PY_TPFLAGS_UNICODE_SUBCLASS: c_ulong = 1 << 28;
const PY_TPFLAGS_DICT_SUBCLASS: c_ulong = 1 << 29;
const PY_TPFLAGS_BASE_EXC_SUBCLASS: c_ulong = 1 << 30;
const PY_TPFLAGS_TYPE_SUBCLASS: c_ulong = 1 << 31;

static mut ACTUAL_PYBASEOBJECT_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYBOOL_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYBYTEARRAY_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYBYTES_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYDICT_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYFLOAT_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYLIST_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYLONG_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYMODULE_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYTUPLE_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYTYPE_TYPE: *mut PyTypeObject = ptr::null_mut();
static mut ACTUAL_PYUNICODE_TYPE: *mut PyTypeObject = ptr::null_mut();

static mut ACTUAL_PYNONESTRUCT: *mut PyObject = ptr::null_mut();
static mut ACTUAL_PYFALSESTRUCT: *mut PyObject = ptr::null_mut();
static mut ACTUAL_PYTRUESTRUCT: *mut PyObject = ptr::null_mut();
static mut ACTUAL_PYNOTIMPLEMENTEDSTRUCT: *mut PyObject = ptr::null_mut();

fn compute_exported_type_flags(actual: *mut PyTypeObject) -> c_ulong {
    let actual = normalize_type_ptr(actual);
    let ty = unsafe { &*actual };
    let mut flags = ty.slots.flags.bits();

    let _ = try_with_current_vm(|vm| {
        if ty.fast_issubclass(vm.ctx.types.int_type) {
            flags |= PY_TPFLAGS_LONG_SUBCLASS;
        }
        if ty.fast_issubclass(vm.ctx.types.list_type) {
            flags |= PY_TPFLAGS_LIST_SUBCLASS;
        }
        if ty.fast_issubclass(vm.ctx.types.tuple_type) {
            flags |= PY_TPFLAGS_TUPLE_SUBCLASS;
        }
        if ty.fast_issubclass(vm.ctx.types.bytes_type) {
            flags |= PY_TPFLAGS_BYTES_SUBCLASS;
        }
        if ty.fast_issubclass(vm.ctx.types.str_type) {
            flags |= PY_TPFLAGS_UNICODE_SUBCLASS;
        }
        if ty.fast_issubclass(vm.ctx.types.dict_type) {
            flags |= PY_TPFLAGS_DICT_SUBCLASS;
        }
        if ty.fast_issubclass(vm.ctx.exceptions.base_exception_type) {
            flags |= PY_TPFLAGS_BASE_EXC_SUBCLASS;
        }
        if ty.fast_issubclass(vm.ctx.types.type_type) {
            flags |= PY_TPFLAGS_TYPE_SUBCLASS;
        }
    });

    flags
}

#[unsafe(export_name = "PyBaseObject_Type")]
static mut PYBASEOBJECT_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "PyBool_Type")]
static mut PYBOOL_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "PyByteArray_Type")]
static mut PYBYTEARRAY_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "PyBytes_Type")]
static mut PYBYTES_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "PyDict_Type")]
static mut PYDICT_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "PyFloat_Type")]
static mut PYFLOAT_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "PyList_Type")]
static mut PYLIST_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "PyLong_Type")]
static mut PYLONG_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "PyModule_Type")]
static mut PYMODULE_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "PyTuple_Type")]
static mut PYTUPLE_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "PyType_Type")]
static mut PYTYPE_TYPE_EXPORT: CApiTypeObjectPrefix = CApiTypeObjectPrefix {
    ob_base: CApiVarObjectHeader {
        ob_base: CApiObjectHeader {
            ob_refcnt: 1,
            ob_type: ptr::null_mut(),
        },
        ob_size: 0,
    },
    tp_name: ptr::null(),
    tp_basicsize: 0,
    tp_itemsize: 0,
    tp_dealloc: ptr::null_mut(),
    tp_vectorcall_offset: 0,
    tp_getattr: ptr::null_mut(),
    tp_setattr: ptr::null_mut(),
    tp_as_async: ptr::null_mut(),
    tp_repr: ptr::null_mut(),
    tp_as_number: ptr::null_mut(),
    tp_as_sequence: ptr::null_mut(),
    tp_as_mapping: ptr::null_mut(),
    tp_hash: ptr::null_mut(),
    tp_call: ptr::null_mut(),
    tp_str: ptr::null_mut(),
    tp_getattro: ptr::null_mut(),
    tp_setattro: ptr::null_mut(),
    tp_as_buffer: ptr::null_mut(),
    tp_flags: 0,
};
#[unsafe(export_name = "PyUnicode_Type")]
static mut PYUNICODE_TYPE_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};

#[unsafe(export_name = "_Py_NoneStruct")]
static mut PYNONESTRUCT_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "_Py_FalseStruct")]
static mut PYFALSESTRUCT_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "_Py_TrueStruct")]
static mut PYTRUESTRUCT_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};
#[unsafe(export_name = "_Py_NotImplementedStruct")]
static mut PYNOTIMPLEMENTEDSTRUCT_EXPORT: ExportedStaticObject = ExportedStaticObject {
    ob_refcnt: 1,
    ob_type: ptr::null_mut(),
};

#[allow(static_mut_refs)]
pub(crate) unsafe fn init_exported_builtin_objects(ctx: &Context) {
    unsafe {
        let object_type = ctx.types.object_type.to_owned();
        let bool_type = ctx.types.bool_type.to_owned();
        let bytearray_type = ctx.types.bytearray_type.to_owned();
        let bytes_type = ctx.types.bytes_type.to_owned();
        let dict_type = ctx.types.dict_type.to_owned();
        let float_type = ctx.types.float_type.to_owned();
        let list_type = ctx.types.list_type.to_owned();
        let int_type = ctx.types.int_type.to_owned();
        let module_type = ctx.types.module_type.to_owned();
        let tuple_type = ctx.types.tuple_type.to_owned();
        let type_type = ctx.types.type_type.to_owned();
        let type_type_flags = type_type.slots.flags.bits() | PY_TPFLAGS_TYPE_SUBCLASS;
        let str_type = ctx.types.str_type.to_owned();
        let none: PyObjectRef = ctx.none.to_owned().into();
        let false_value: PyObjectRef = ctx.false_value.to_owned().into();
        let true_value: PyObjectRef = ctx.true_value.to_owned().into();
        let not_implemented: PyObjectRef = ctx.not_implemented.to_owned().into();

        ACTUAL_PYBASEOBJECT_TYPE =
            normalize_type_ptr(object_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYBOOL_TYPE = normalize_type_ptr(bool_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYBYTEARRAY_TYPE =
            normalize_type_ptr(bytearray_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYBYTES_TYPE =
            normalize_type_ptr(bytes_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYDICT_TYPE = normalize_type_ptr(dict_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYFLOAT_TYPE =
            normalize_type_ptr(float_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYLIST_TYPE = normalize_type_ptr(list_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYLONG_TYPE = normalize_type_ptr(int_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYMODULE_TYPE =
            normalize_type_ptr(module_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYTUPLE_TYPE =
            normalize_type_ptr(tuple_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYTYPE_TYPE = normalize_type_ptr(type_type.as_object().as_raw().cast_mut().cast());
        ACTUAL_PYUNICODE_TYPE =
            normalize_type_ptr(str_type.as_object().as_raw().cast_mut().cast());

        ACTUAL_PYNONESTRUCT = none.as_raw().cast_mut();
        ACTUAL_PYFALSESTRUCT = false_value.as_raw().cast_mut();
        ACTUAL_PYTRUESTRUCT = true_value.as_raw().cast_mut();
        ACTUAL_PYNOTIMPLEMENTEDSTRUCT = not_implemented.as_raw().cast_mut();

        let retained = retained_builtin_objects();
        let mut retained = retained.lock().unwrap();
        retained.clear();
        retained.extend([
            object_type.into(),
            bool_type.into(),
            bytearray_type.into(),
            bytes_type.into(),
            dict_type.into(),
            float_type.into(),
            list_type.into(),
            int_type.into(),
            module_type.into(),
            tuple_type.into(),
            type_type.into(),
            str_type.into(),
            none,
            false_value,
            true_value,
            not_implemented,
        ]);

        let pytype_export = ptr::addr_of_mut!(PYTYPE_TYPE_EXPORT).cast::<PyTypeObject>();
        PYTYPE_TYPE_EXPORT.ob_base.ob_base.ob_type = pytype_export;
        PYTYPE_TYPE_EXPORT.tp_flags = type_type_flags;
        for exported in [
            ptr::addr_of_mut!(PYBASEOBJECT_TYPE_EXPORT),
            ptr::addr_of_mut!(PYBOOL_TYPE_EXPORT),
            ptr::addr_of_mut!(PYBYTEARRAY_TYPE_EXPORT),
            ptr::addr_of_mut!(PYBYTES_TYPE_EXPORT),
            ptr::addr_of_mut!(PYDICT_TYPE_EXPORT),
            ptr::addr_of_mut!(PYFLOAT_TYPE_EXPORT),
            ptr::addr_of_mut!(PYLIST_TYPE_EXPORT),
            ptr::addr_of_mut!(PYLONG_TYPE_EXPORT),
            ptr::addr_of_mut!(PYMODULE_TYPE_EXPORT),
            ptr::addr_of_mut!(PYTUPLE_TYPE_EXPORT),
            ptr::addr_of_mut!(PYUNICODE_TYPE_EXPORT),
        ] {
            (*exported).ob_type = pytype_export;
        }

        PYNONESTRUCT_EXPORT.ob_type = ctx
            .none
            .class()
            .as_object()
            .as_raw()
            .cast_mut()
            .cast();
        PYFALSESTRUCT_EXPORT.ob_type = ptr::addr_of_mut!(PYBOOL_TYPE_EXPORT).cast::<PyTypeObject>();
        PYTRUESTRUCT_EXPORT.ob_type = ptr::addr_of_mut!(PYBOOL_TYPE_EXPORT).cast::<PyTypeObject>();
        PYNOTIMPLEMENTEDSTRUCT_EXPORT.ob_type = ctx
            .types
            .not_implemented_type
            .as_object()
            .as_raw()
            .cast_mut()
            .cast();
    }
}

unsafe fn create_wrapper(actual: *mut PyObject, min_size: usize) -> *mut PyObject {
    let header_size = core::mem::size_of::<CApiObjectHeader>();
    let is_type = unsafe { (&*actual).downcast_ref::<PyType>().is_some() };
    let type_prefix_size = core::mem::size_of::<CApiTypeObjectPrefix>();
    let size = if is_type {
        min_size.max(type_prefix_size)
    } else {
        min_size.max(header_size)
    };
    let align = if is_type {
        core::mem::align_of::<CApiTypeObjectPrefix>()
    } else {
        core::mem::align_of::<CApiObjectHeader>()
    };
    let layout = Layout::from_size_align(size, align).expect("valid wrapper layout");
    let wrapper = unsafe { alloc_zeroed(layout) };
    if wrapper.is_null() {
        return core::ptr::null_mut();
    }

    let actual_type = unsafe {
        (*actual)
            .class()
            .as_object()
            .as_raw()
            .cast_mut()
            .cast::<PyTypeObject>()
    };
    let wrapper = wrapper.cast::<CApiObjectHeader>();
    unsafe {
        (*wrapper).ob_refcnt = 1;
        (*wrapper).ob_type = exported_type_handle(actual_type);
        if is_type {
            let type_wrapper = wrapper.cast::<CApiTypeObjectPrefix>();
            (*type_wrapper).ob_base.ob_size = 0;
            (*type_wrapper).tp_flags = compute_exported_type_flags(actual_type);
        }
    }

    let wrapper_ptr = wrapper.cast::<PyObject>();
    unsafe {
        core::mem::forget((&*actual).to_owned());
    }
    let mut maps = wrapper_maps().lock().unwrap();
    maps.inner_to_wrapper
        .insert(actual as usize, wrapper_ptr as usize);
    maps.wrapper_to_inner
        .insert(wrapper_ptr as usize, actual as usize);
    wrapper_ptr
}

pub(crate) unsafe fn exported_object_wrapper(
    actual: *mut PyObject,
    min_size: usize,
) -> *mut PyObject {
    let maps = wrapper_maps().lock().unwrap();
    if let Some(wrapper) = maps.inner_to_wrapper.get(&(actual as usize)).copied() {
        wrapper as *mut PyObject
    } else {
        drop(maps);
        unsafe { create_wrapper(actual, min_size) }
    }
}

pub(crate) unsafe fn wrapper_refcnt(op: *mut PyObject) -> Option<isize> {
    let maps = wrapper_maps().lock().unwrap();
    maps.wrapper_to_inner
        .contains_key(&(op as usize))
        .then(|| unsafe { (*(op as *mut CApiObjectHeader)).ob_refcnt })
}

pub(crate) unsafe fn incref_wrapper(op: *mut PyObject) -> bool {
    let maps = wrapper_maps().lock().unwrap();
    if !maps.wrapper_to_inner.contains_key(&(op as usize)) {
        return false;
    }
    drop(maps);
    unsafe {
        let header = op as *mut CApiObjectHeader;
        (*header).ob_refcnt += 1;
    }
    true
}

pub(crate) unsafe fn decref_wrapper(op: *mut PyObject) -> bool {
    let inner = {
        let maps = wrapper_maps().lock().unwrap();
        let Some(inner) = maps.wrapper_to_inner.get(&(op as usize)).copied() else {
            return false;
        };
        inner as *mut PyObject
    };

    if unsafe { (&*inner).downcast_ref::<PyType>() }.is_some() {
        return true;
    }

    let should_free = unsafe {
        let header = op as *mut CApiObjectHeader;
        (*header).ob_refcnt -= 1;
        (*header).ob_refcnt == 0
    };

    if should_free {
        let mut maps = wrapper_maps().lock().unwrap();
        maps.wrapper_to_inner.remove(&(op as usize));
        maps.inner_to_wrapper.remove(&(inner as usize));
        drop(maps);

        unsafe {
            let _ = rustpython_vm::PyObjectRef::from_raw(
                core::ptr::NonNull::new_unchecked(inner),
            );
        }
    }

    true
}

#[inline]
pub(crate) unsafe fn exported_type_handle(actual: *mut PyTypeObject) -> *mut PyTypeObject {
    let actual = normalize_type_ptr(actual);
    unsafe {
        if actual == ACTUAL_PYBASEOBJECT_TYPE {
            ptr::addr_of_mut!(PYBASEOBJECT_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYBOOL_TYPE {
            ptr::addr_of_mut!(PYBOOL_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYBYTEARRAY_TYPE {
            ptr::addr_of_mut!(PYBYTEARRAY_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYBYTES_TYPE {
            ptr::addr_of_mut!(PYBYTES_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYDICT_TYPE {
            ptr::addr_of_mut!(PYDICT_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYFLOAT_TYPE {
            ptr::addr_of_mut!(PYFLOAT_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYLIST_TYPE {
            ptr::addr_of_mut!(PYLIST_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYLONG_TYPE {
            ptr::addr_of_mut!(PYLONG_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYMODULE_TYPE {
            ptr::addr_of_mut!(PYMODULE_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYTUPLE_TYPE {
            ptr::addr_of_mut!(PYTUPLE_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYTYPE_TYPE {
            ptr::addr_of_mut!(PYTYPE_TYPE_EXPORT).cast()
        } else if actual == ACTUAL_PYUNICODE_TYPE {
            ptr::addr_of_mut!(PYUNICODE_TYPE_EXPORT).cast()
        } else {
            unsafe { exported_object_wrapper(actual.cast(), core::mem::size_of::<CApiTypeObjectPrefix>()) }
                .cast()
        }
    }
}

pub(crate) unsafe fn exported_type_flags(exported: *mut PyTypeObject) -> Option<c_ulong> {
    let exported = normalize_type_ptr(exported);
    if exported == ptr::addr_of_mut!(PYTYPE_TYPE_EXPORT).cast() {
        return Some(unsafe { PYTYPE_TYPE_EXPORT.tp_flags });
    }
    let maps = wrapper_maps().lock().unwrap();
    let inner = maps.wrapper_to_inner.get(&(exported as usize)).copied()?;
    drop(maps);
    Some(compute_exported_type_flags(
        (inner as *mut PyObject).cast::<PyTypeObject>(),
    ))
}

#[inline]
pub(crate) unsafe fn resolve_type_handle(exported: *mut PyTypeObject) -> *mut PyTypeObject {
    let exported = normalize_type_ptr(exported);
    unsafe {
        if exported == ptr::addr_of_mut!(PYBASEOBJECT_TYPE_EXPORT).cast() {
            ACTUAL_PYBASEOBJECT_TYPE
        } else if exported == ptr::addr_of_mut!(PYBOOL_TYPE_EXPORT).cast() {
            ACTUAL_PYBOOL_TYPE
        } else if exported == ptr::addr_of_mut!(PYBYTEARRAY_TYPE_EXPORT).cast() {
            ACTUAL_PYBYTEARRAY_TYPE
        } else if exported == ptr::addr_of_mut!(PYBYTES_TYPE_EXPORT).cast() {
            ACTUAL_PYBYTES_TYPE
        } else if exported == ptr::addr_of_mut!(PYDICT_TYPE_EXPORT).cast() {
            ACTUAL_PYDICT_TYPE
        } else if exported == ptr::addr_of_mut!(PYFLOAT_TYPE_EXPORT).cast() {
            ACTUAL_PYFLOAT_TYPE
        } else if exported == ptr::addr_of_mut!(PYLIST_TYPE_EXPORT).cast() {
            ACTUAL_PYLIST_TYPE
        } else if exported == ptr::addr_of_mut!(PYLONG_TYPE_EXPORT).cast() {
            ACTUAL_PYLONG_TYPE
        } else if exported == ptr::addr_of_mut!(PYMODULE_TYPE_EXPORT).cast() {
            ACTUAL_PYMODULE_TYPE
        } else if exported == ptr::addr_of_mut!(PYTUPLE_TYPE_EXPORT).cast() {
            ACTUAL_PYTUPLE_TYPE
        } else if exported == ptr::addr_of_mut!(PYTYPE_TYPE_EXPORT).cast() {
            ACTUAL_PYTYPE_TYPE
        } else if exported == ptr::addr_of_mut!(PYUNICODE_TYPE_EXPORT).cast() {
            ACTUAL_PYUNICODE_TYPE
        } else {
            let maps = wrapper_maps().lock().unwrap();
            maps.wrapper_to_inner
                .get(&(exported as usize))
                .copied()
                .map(|ptr| normalize_type_ptr(ptr as *mut PyTypeObject))
                .unwrap_or(exported)
        }
    }
}

#[inline]
pub(crate) unsafe fn exported_object_handle(actual: *mut PyObject) -> *mut PyObject {
    unsafe {
        if actual.is_null() {
            return ptr::null_mut();
        }
        if actual == ptr::addr_of_mut!(PYNONESTRUCT_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYFALSESTRUCT_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYTRUESTRUCT_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYNOTIMPLEMENTEDSTRUCT_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYBASEOBJECT_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYBOOL_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYBYTEARRAY_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYBYTES_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYDICT_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYFLOAT_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYLIST_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYLONG_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYMODULE_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYTUPLE_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYTYPE_TYPE_EXPORT).cast()
            || actual == ptr::addr_of_mut!(PYUNICODE_TYPE_EXPORT).cast()
        {
            actual
        } else if wrapper_maps()
            .lock()
            .unwrap()
            .wrapper_to_inner
            .contains_key(&(actual as usize))
        {
            actual
        } else if actual == ACTUAL_PYNONESTRUCT {
            ptr::addr_of_mut!(PYNONESTRUCT_EXPORT).cast()
        } else if actual == ACTUAL_PYFALSESTRUCT {
            ptr::addr_of_mut!(PYFALSESTRUCT_EXPORT).cast()
        } else if actual == ACTUAL_PYTRUESTRUCT {
            ptr::addr_of_mut!(PYTRUESTRUCT_EXPORT).cast()
        } else if actual == ACTUAL_PYNOTIMPLEMENTEDSTRUCT {
            ptr::addr_of_mut!(PYNOTIMPLEMENTEDSTRUCT_EXPORT).cast()
        } else {
            let actual_class = normalize_type_ptr(
                (*actual)
                    .class()
                    .as_object()
                    .as_raw()
                    .cast_mut()
                    .cast::<PyTypeObject>(),
            );
            let is_type_object = actual_class == ACTUAL_PYTYPE_TYPE
                || (!ACTUAL_PYTYPE_TYPE.is_null() && (&*actual_class).is_subtype(&*ACTUAL_PYTYPE_TYPE));
            if is_type_object {
                let exported = exported_type_handle(actual.cast());
                if exported == actual.cast() {
                    exported_object_wrapper(actual, core::mem::size_of::<usize>() * 2)
                } else {
                    exported.cast()
                }
            } else {
                let maps = wrapper_maps().lock().unwrap();
                if let Some(wrapper) = maps.inner_to_wrapper.get(&(actual as usize)).copied() {
                    wrapper as *mut PyObject
                } else {
                    drop(maps);
                    exported_object_wrapper(actual, core::mem::size_of::<usize>() * 2)
                }
            }
        }
    }
}

#[inline]
pub(crate) unsafe fn resolve_object_handle(exported: *mut PyObject) -> *mut PyObject {
    unsafe {
        if exported == ptr::addr_of_mut!(PYNONESTRUCT_EXPORT).cast() {
            ACTUAL_PYNONESTRUCT
        } else if exported == ptr::addr_of_mut!(PYFALSESTRUCT_EXPORT).cast() {
            ACTUAL_PYFALSESTRUCT
        } else if exported == ptr::addr_of_mut!(PYTRUESTRUCT_EXPORT).cast() {
            ACTUAL_PYTRUESTRUCT
        } else if exported == ptr::addr_of_mut!(PYNOTIMPLEMENTEDSTRUCT_EXPORT).cast() {
            ACTUAL_PYNOTIMPLEMENTEDSTRUCT
        } else if exported == ptr::addr_of_mut!(PYBASEOBJECT_TYPE_EXPORT).cast() {
            ACTUAL_PYBASEOBJECT_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYBOOL_TYPE_EXPORT).cast() {
            ACTUAL_PYBOOL_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYBYTEARRAY_TYPE_EXPORT).cast() {
            ACTUAL_PYBYTEARRAY_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYBYTES_TYPE_EXPORT).cast() {
            ACTUAL_PYBYTES_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYDICT_TYPE_EXPORT).cast() {
            ACTUAL_PYDICT_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYFLOAT_TYPE_EXPORT).cast() {
            ACTUAL_PYFLOAT_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYLIST_TYPE_EXPORT).cast() {
            ACTUAL_PYLIST_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYLONG_TYPE_EXPORT).cast() {
            ACTUAL_PYLONG_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYMODULE_TYPE_EXPORT).cast() {
            ACTUAL_PYMODULE_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYTUPLE_TYPE_EXPORT).cast() {
            ACTUAL_PYTUPLE_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYTYPE_TYPE_EXPORT).cast() {
            ACTUAL_PYTYPE_TYPE.cast()
        } else if exported == ptr::addr_of_mut!(PYUNICODE_TYPE_EXPORT).cast() {
            ACTUAL_PYUNICODE_TYPE.cast()
        } else {
            let maps = wrapper_maps().lock().unwrap();
            maps.wrapper_to_inner
                .get(&(exported as usize))
                .copied()
                .map(|ptr| ptr as *mut PyObject)
                .unwrap_or(exported)
        }
    }
}
