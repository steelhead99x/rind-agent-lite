---
name: contribute
description: Use when preparing a rind change to send upstream.
---

# Contribute

Read [`AGENTS.md`](https://github.com/kixel-ai/rind/blob/acc61b3892ceb6afecfa96c94132bf6e35bbf1fb/AGENTS.md)
and
[`CONTRIBUTING.md`](https://github.com/kixel-ai/rind/blob/acc61b3892ceb6afecfa96c94132bf6e35bbf1fb/CONTRIBUTING.md)
before opening a pull request. Pull requests target `main`.

## Upstream gate

Nothing is submitted upstream (no pull request, issue, comment, or push
to kixel-ai/rind or any other upstream) until a second independent
scored review of the head commit scores 9 or 10 with zero high, medium,
or low findings. After any fix, the review runs again on the new head.
The score, the reviewer, the head SHA, and the findings are recorded in
the pull request body. An explicit OK is still required on top of that
review.

Run rind's checks as its docs state them. Sending that work upstream
follows this gate.

1. Branch from current `main`. One problem per branch.
2. Keep the `low` / `mid` / `high` layering. Shape a public API so
   misuse fails to compile.
3. Add the tests in [test](../test/SKILL.md) and, for a crypto change,
   the checks in [crypto-review](../crypto-review/SKILL.md).
4. Run `just check` on the head you will push. rind's `CONTRIBUTING.md`
   says to run `just ci` before every push.
5. Use a conventional-commit subject. Do not commit build artifacts.
6. Before opening the pull request, check `UPSTREAM.md`. A new
   workaround for a tool rind does not own gets an entry there, named
   from the workaround's comment.

The pull request body has Problem, Approach, Impact, Alternatives, and
Follow-ups, as rind's `CONTRIBUTING.md` lists. Impact names each public
API change. A breaking change puts `!` in the title and says how to
migrate.

A pull request that touches platform-workflow inputs also gets the
advisory runs that file lists. Fix a red one before merging. `mutants`
does not run on a pull request. Mutate a diff with
`admin/mutants --in-diff`.
