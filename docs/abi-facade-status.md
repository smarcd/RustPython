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
