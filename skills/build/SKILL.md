---
name: build
description: Use when building the rind workspace.
---

# Build

From a rind checkout:

```shell
git submodule update --init thirdparty/cavp thirdparty/cctv thirdparty/wycheproof
nix develop
cargo build -p rind
```

`nix develop` provides the toolchain CI uses. Without Nix, a stable
Rust toolchain and `just` are enough to compile.

`just fix` formats the branch and applies the lint fixes `sh/dispatch.sh`
selects. The recipes are in the `justfile`.

One backend compiles per target. A missing CPU feature is an error that
names the feature. Do not add a runtime fallback. The layering is
`low`, `mid`, and `high`, as `ARCHITECTURE.md` describes.
