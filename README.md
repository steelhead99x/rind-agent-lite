# rind-agent-lite

Short skills for working on [rind](https://github.com/kixel-ai/rind), a
pure-Rust cryptography library. `moq-dev/rind` redirects there.

Each skill is a `skills/<name>/SKILL.md` file. The frontmatter has `name`
and `description`. The description starts with "Use when".

Where a skill and rind disagree, rind wins. The commands below were
checked against
[`acc61b3`](https://github.com/kixel-ai/rind/commit/acc61b3892ceb6afecfa96c94132bf6e35bbf1fb).

## Skills

| Skill | Use it when |
| --- | --- |
| [`rind`](skills/rind/SKILL.md) | You need what rind is and which docs to read first. |
| [`build`](skills/build/SKILL.md) | You are building the workspace. |
| [`test`](skills/test/SKILL.md) | You are running rind's tests. |
| [`bench`](skills/bench/SKILL.md) | You are measuring rind on a host or a board. |
| [`crypto-review`](skills/crypto-review/SKILL.md) | You are reviewing a cryptography change. |
| [`contribute`](skills/contribute/SKILL.md) | You are preparing a change to send upstream. |

## License

Apache-2.0, ISC, or MIT-0, at your option. See [`LICENSE`](LICENSE).
