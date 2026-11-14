---
name: rind
description: Use when you need what rind is and which docs to read first.
---

# rind

[rind](https://github.com/kixel-ai/rind) is a pure-Rust cryptography
library for [MoQ](https://github.com/moq-dev/moq). It is derived from
[graviola](https://github.com/ctz/graviola). `moq-dev/rind` redirects
to kixel-ai/rind.

The repository is one Cargo workspace. `rind` is the library.
`rind-<consumer>` crates adapt it to other projects, including
`rind-rustls`, `rind-ring`, and `rind-aws-lc-rs`. Bench, fuzz, cpuid,
and timing crates are unpublished.

Read these before changing code:

- [`AGENTS.md`](https://github.com/kixel-ai/rind/blob/acc61b3892ceb6afecfa96c94132bf6e35bbf1fb/AGENTS.md)
- [`ARCHITECTURE.md`](https://github.com/kixel-ai/rind/blob/acc61b3892ceb6afecfa96c94132bf6e35bbf1fb/ARCHITECTURE.md)
- [`PLATFORMS.md`](https://github.com/kixel-ai/rind/blob/acc61b3892ceb6afecfa96c94132bf6e35bbf1fb/PLATFORMS.md)
- [`CONTRIBUTING.md`](https://github.com/kixel-ai/rind/blob/acc61b3892ceb6afecfa96c94132bf6e35bbf1fb/CONTRIBUTING.md)

Also read any `AGENTS.md` nested under the files you change. Where this
skill and those files disagree, rind wins.
