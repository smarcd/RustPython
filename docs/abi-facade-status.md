# ABI Facade Status

## Maintained latest-main baseline

- branch: `abi-facade-pyo3-maintained`
- commit: `632a8c037ddf10ff2e15de4e3dcfd08d4062fba2`
- upstream base: current `RustPython/main` as of the forward-port checkpoint
- status: latest-main adaptation complete and re-verified

## Verified unchanged downstream packages

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

## Current interpretation

- the maintained branch now compiles and runs on latest `main`
- the current ABI facade is good enough for multiple unchanged real PyO3 packages
- the remaining work is broader ABI completeness, not basic loader viability

## Ranked ABI backlog

### Tier 1: broad pristine PyO3 compatibility
- `PyType_FromSpec` slot coverage in `crates/capi/src/object.rs`
- `PyType_GetSlot` coverage in `crates/capi/src/object.rs`
- richer buffer support in `crates/capi/src/pybuffer.rs`

### Tier 2: common ecosystem compatibility
- unicode/encoding APIs in `crates/capi/src/unicodeobject.rs`
- lifecycle/finalization semantics in `crates/capi/src/pylifecycle.rs`

### Tier 3: architectural follow-up
- exported builtin/type handle model in `crates/capi/src/handles.rs`
