---
name: test
description: Use when running rind's tests.
---

# Test

From the rind root, with the dev shell active:

```shell
just check
cargo test -p rind
just ci
```

`just check` lints, compiles, and tests what the branch changed, plus
crates that depend on those changes. `just ci` is the same gate with
missing tools treated as failures. rind's `CONTRIBUTING.md` says to run
`just ci` before every push. `just gate` runs that gate over every crate.

Initialize `thirdparty/cavp`, `thirdparty/cctv`, and
`thirdparty/wycheproof` first, or the gate stops early.

Other architectures, from `TESTING.md` under "Running the tools":

```shell
just armv7
just qemu <shell>
just wasm
just miri
just portable
```

`just qemu` shells are `s390x`, `riscv64`, `riscv64-musl`,
`loongarch64`, `armv6`, `armv6-softfloat`, `powerpc64le`, `powerpc64`,
`powerpc`, and `i686`.

`SLOW_TESTS=1 cargo test` adds the large vector sets. Constant-time
checks are `admin/ctgrind` in the `.#ctgrind` shell, as `TESTING.md`
describes. Do not turn on `--all-features` locally just to reach them:
that enables `__ctgrind`, which needs valgrind headers.

Gate a test that uses aws-lc with
`#[cfg(not(any(miri, target_family = "wasm")))]`. aws-lc does not build
for wasm, and Miri cannot run C.
