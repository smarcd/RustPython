# ABI Facade Status

## Maintained ABI facade baseline

- branch: `abi-facade-pyo3-maintained`
- commit: `c0357af501688c9aa239b7ee181f964d593a7088`
- upstream base: current `RustPython/main` as of the forward-port checkpoint
- status: maintained branch refreshed and re-verified

## Verified downstream packages

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

### Unicode / lifecycle / finalization
- unicode and encoding APIs in `crates/capi/src/unicodeobject.rs`
- lifecycle and finalization semantics in `crates/capi/src/pylifecycle.rs`

### Package matrix expansion
- add the next non-`numpy` pristine PyO3 package lane when local artifacts can be recreated cleanly
- expand the matrix toward `pydantic-core`, `orjson`, or `msgspec` as needed
