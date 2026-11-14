---
name: crypto-review
description: Use when reviewing a cryptography change in rind.
---

# Crypto review

Follow
[`REVIEW.md`](https://github.com/kixel-ai/rind/blob/acc61b3892ceb6afecfa96c94132bf6e35bbf1fb/REVIEW.md).
The title, body, and commits are claims to check. Read the diff against
the base the pull request targets.

On every path the change touches:

- Trace each secret. A secret-dependent branch, memory index, or
  variable-time instruction is a finding. A new secret needs a
  `low::ct::secret_slice` mark or an `admin/ctgrind` test.
- Caller input must not panic. Check `unwrap`, `expect`, indexing,
  overflow, and `assert`.
- A new primitive needs Wycheproof vectors where they exist, and a
  differential test against aws-lc-rs where aws-lc-rs has it. Include
  empty, zero, maximal, and malformed inputs.
- A key type wipes its whole value on drop, public parts included,
  through `low::zeroise` or `low::zeroise_value`, which run only under
  the `wipe` feature. Do not wipe a digest context, a decrypted
  message, or scratch that no private key reaches. `rind-fips` is the
  exception `REVIEW.md` names.
- One backend compiles per target. A missing CPU feature is an error
  that names the feature.
- An adapter lives in its own `rind-<consumer>` crate and matches the
  upstream provider's API. A divergence is a bug.

A number that will be published comes from `admin/bench-report`, not
from a bare `cargo bench`. Record the host and the date.
