# ABI Facade Status

## Maintained ABI facade baseline

- branch: `abi-facade-pyo3-maintained`
- commit: `c0357af501688c9aa239b7ee181f964d593a7088`
- upstream base: current `RustPython/main` as of the forward-port checkpoint
- status: maintained branch refreshed and re-verified
- interpretation: the maintained branch is already viable on current `main`; the remaining work is ABI completeness, not basic loader viability

## Verified downstream packages

- verification provenance: maintained downstream verifier rerun on `2026-04-22`
  - command: `RUSTPYTHON_ABI_ROOT=/Users/sunny/work/codepod/rustpython-abi-facade BLAKE3_SITE=/tmp/blake3-wheel-site-abi3 JITER_SITE=/private/tmp/jiter-abi-venv/lib/python3.14/site-packages RPDS_SITE=/private/tmp/rpds-abi-venv/lib/python3.14/site-packages /Users/sunny/work/codepod/pyo3-rustpython/.local/abi-facade/verify_pyo3_downstream.sh`
  - package counts below are from that run against the maintained worktree at `c0357af501688c9aa239b7ee181f964d593a7088`
- `rpds`: package-owned Python tests green
  - `test_hash_trie_map.py`: `ran=59 skipped=0 failed=0`
  - `test_hash_trie_set.py`: `ran=21 skipped=0 failed=0`
  - `test_list.py`: `ran=19 skipped=0 failed=0`
  - `test_queue.py`: `ran=17 skipped=0 failed=0`
  - `test_stack.py`: `ran=16 skipped=0 failed=0`
  - final runner status: `package-manual-tests-ok`
- `jiter`: package-owned Python tests green except one intentional skip in the current manual runner
  - `ran=32 skipped=1 failed=0`
  - final runner status: `jiter-manual-tests-ok`
- `blake3`: package-owned Python tests green except the remaining `numpy`-dependent skip
  - `ran=26 skipped=1 failed=0`
  - skipped test: `test_strided_array_fails`
  - exact buffer contract exercised by that case: a 1-D readable/writable strided view with `format='B'`, `itemsize=1`, `shape=(2,)`, `strides=(2,)`, `suboffsets=()`, and `readonly=False`
  - direct reproduction against the facade with a sliced `memoryview` yields `BufferError: non-contiguous buffers are not yet supported`, which matches the package expectation for a non-contiguous input
  - sharp blocker: the remaining skipped test is still a `numpy` lane problem, not a missing strided-buffer contract in `pybuffer.rs`
  - final runner status: `blake3-manual-tests-ok`

## Verification policy

Downstream verification helpers are local-only and are not part of the RustPython PR.

## Latest Landed Slot Families

- heap type slot coverage in `crates/capi/src/object.rs`
- unsupported-slot panic avoidance in `crates/capi/src/object.rs`
- arithmetic slot coverage in `crates/capi/src/object.rs`
- call and descriptor slot coverage in `crates/capi/src/object.rs`
- sequence and mapping mutation slots in `crates/capi/src/object.rs`
- sequence operator slots in `crates/capi/src/object.rs`

## Remaining Backlog

### Heap type / slot completeness
- finish the remaining `PyType_FromSpec` and `PyType_GetSlot` mappings in `crates/capi/src/object.rs`
- close out any unhandled heap-type finalization edge cases as they are proven by downstream packages

### Buffer / array / `numpy`
- richer buffer support in `crates/capi/src/pybuffer.rs`
- keep the `numpy`-dependent gaps explicit until a real package lane proves the next required surface
- the current `blake3` strided-array case is now a documented packaging/lane blocker, not an undefined buffer-semantic gap

### Unicode / lifecycle / finalization
- unicode and encoding APIs in `crates/capi/src/unicodeobject.rs`
- lifecycle and finalization semantics in `crates/capi/src/pylifecycle.rs`

### Package matrix expansion
- add the next non-`numpy` pristine PyO3 package lane when local artifacts can be recreated cleanly
- expand the matrix toward additional real-world PyO3 packages as downstream proof points require it

## Additional Package Lane: `jsonschema-py`

- package: `jsonschema-py` from upstream `jsonschema-rs` `master` (`89848e0e0c79093516e3c0bf9e670df8d45d9b07`)
- unchanged source confirmation:
  - package source was exercised from a clean temporary worktree at `/tmp/jsonschema-py-abi-facade`
  - no package files were edited
  - local-only build glue used `/tmp/pyo3-cpython-abi3.config` and a local venv at `/tmp/jsonschema-abi-venv`
- build outcome:
  - unchanged `jsonschema-py` wheel built successfully against pristine `pyo3 0.28.3`
  - command:
    - `PYO3_CONFIG_FILE=/tmp/pyo3-cpython-abi3.config maturin build --strip false --interpreter /tmp/jsonschema-abi-venv/bin/python --out /tmp/jsonschema-wheels -m /tmp/jsonschema-py-abi-facade/crates/jsonschema-py/Cargo.toml`
- test/import outcome:
  - import under the maintained RustPython binary failed immediately with `ImportError: dlopen failed`
  - command:
    - `PYTHONPATH=/tmp/jsonschema-abi-venv/lib/python3.14/site-packages /Users/sunny/work/codepod/rustpython-abi-facade/target/debug/rustpython -c 'import jsonschema_rs; print(jsonschema_rs.__all__[:5]); print(jsonschema_rs.is_valid({"type":"integer"}, 1))'`
  - blocker class: import-time symbol gap
  - exact missing exported symbols from the maintained binary, derived from `nm -u` on `jsonschema_rs.abi3.so` versus `nm -gU` on `target/debug/rustpython`:
    - `_PyCallable_Check`
    - `_PyDict_Copy`
    - `_PyErr_Clear`
    - `_PyFloat_AsDouble`
    - `_PyFloat_Type`
    - `_PyIter_Next`
    - `_PyList_GetSlice`
    - `_PyLong_AsLongLong`
    - `_PyModule_NewObject`
    - `_PyNumber_Long`
    - `_PyUnicode_InternFromString`
    - `_Py_NewRef`
    - `__Py_NotImplementedStruct`
- facade/core files changed to support it:
  - none in this task
  - only `docs/abi-facade-status.md` was updated to record the concrete package frontier
