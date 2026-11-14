---
name: bench
description: Use when measuring rind on a host or a board.
---

# Benchmarks

## Host

The published comparison is `admin/bench-report`. It times the
`OPERATIONS` list in that file with:

```text
--warm-up-time 1 --measurement-time 3
```

At
[`acc61b3`](https://github.com/kixel-ai/rind/commit/acc61b3892ceb6afecfa96c94132bf6e35bbf1fb)
that list has 68 entries. Read the list and the criterion arguments
from the commit you measure.

```shell
cargo bench -p rind-bench -- <filter>
admin/bench-report run --host <name> --commit <full-sha> --jobs <n> --target-dir <dir> --out <file>
```

A bare `cargo bench` uses criterion's own timing and is not the
published set. `<name>` must be one of the keys in `HOSTS` inside
`admin/bench-report`. `admin/bench-report summary` compares rind with
the other libraries that file records. How to refresh `BENCHMARKS.md`
is in that file.

Compare a performance change with aws-lc-rs when aws-lc-rs has the
primitive. When `TESTING.md` says it does not, compare with the
reference named there, and record the host and the date.

## Boards

Building a firmware image does not flash it. Ask before any flash,
erase, or unlock. Name the board and the image. Wait for an explicit
yes.

Do not erase. Do not burn efuses. Read the flash at least twice before
a write and keep that image out of git. After the session, read the
flash back and check its sha256 against the backup.

A board number is a measurement of that board. Keep it out of the host
report unless `admin/bench-report` already lists that machine.
