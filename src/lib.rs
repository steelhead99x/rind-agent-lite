//! Checks that this repository names one rind commit, and that every
//! `commit`, `blob`, and `tree` link uses it.
//!
//! The pin is the README link whose text is a hex prefix of the full SHA.
//! The library has no dependencies and does not fetch that commit.

#![forbid(unsafe_code)]

use std::fs;
use std::path::{Path, PathBuf};

/// The rind commit named by a README link.
///
/// `line` is 1-based. `label` is the link text, without a wrapping pair of
/// backticks, and it is a 7 to 40 character hex prefix of `sha`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pin {
    pub owner: String,
    pub repo: String,
    pub sha: String,
    pub label: String,
    pub line: usize,
}

/// One `commit`, `blob`, or `tree` link whose SHA is 40 lowercase hex digits.
///
/// `line` is 1-based. `path` is relative to the repository root and uses `/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    pub path: String,
    pub line: usize,
    pub owner: String,
    pub repo: String,
    pub kind: Kind,
    pub sha: String,
}

/// The GitHub path that carries the commit SHA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Commit,
    Blob,
    Tree,
}

impl Kind {
    /// `commit`, `blob`, or `tree`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Commit => "commit",
            Self::Blob => "blob",
            Self::Tree => "tree",
        }
    }
}

/// The README pin, every full-SHA link, and any problems.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    pub pin: Option<Pin>,
    pub citations: Vec<Citation>,
    pub errors: Vec<String>,
}

const NO_PIN: &str = "README.md: no commit pin; the link text must be a 7 to 40 character hex prefix of the full SHA";

/// Read Markdown under `root` and compare GitHub commit links to the README pin.
#[must_use]
pub fn inspect(root: &Path) -> Report {
    if !root.is_dir() {
        return Report {
            pin: None,
            citations: Vec::new(),
            errors: vec![format!("{}: not a directory", root.display())],
        };
    }
    if !root.join("README.md").is_file() {
        return Report {
            pin: None,
            citations: Vec::new(),
            errors: vec!["README.md: missing".to_string()],
        };
    }

    let mut errors = Vec::new();
    let mut hits = Vec::new();
    for path in markdown_files(root) {
        match fs::read_to_string(&path) {
            Ok(text) => hits.extend(scan_file(root, &path, &text)),
            Err(error) => errors.push(format!("{}: {error}", rel(root, &path))),
        }
    }

    let mut citations = Vec::new();
    let mut pins = Vec::new();
    for hit in &hits {
        if !hit.sha_ok {
            errors.push(format!(
                "{}:{}: {} is not a full lowercase commit SHA",
                hit.path, hit.line, hit.url
            ));
            continue;
        }
        if let Some(label) = &hit.link_text {
            if is_sha_label(label) && !label_is_prefix(label, &hit.sha) {
                errors.push(format!(
                    "{}:{}: link text \"{label}\" is not a prefix of {}",
                    hit.path, hit.line, hit.sha
                ));
            }
        }
        citations.push(Citation {
            path: hit.path.clone(),
            line: hit.line,
            owner: hit.owner.clone(),
            repo: hit.repo.clone(),
            kind: hit.kind,
            sha: hit.sha.clone(),
        });
        if hit.path == "README.md" && hit.kind == Kind::Commit {
            if let Some(label) = pin_label(hit) {
                pins.push(Pin {
                    owner: hit.owner.clone(),
                    repo: hit.repo.clone(),
                    sha: hit.sha.clone(),
                    label: label.to_string(),
                    line: hit.line,
                });
            }
        }
    }

    let pin = apply_pin(&pins, &citations, &mut errors);
    Report {
        pin,
        citations,
        errors,
    }
}

fn apply_pin(pins: &[Pin], citations: &[Citation], errors: &mut Vec<String>) -> Option<Pin> {
    let unique = unique_pins(pins);
    match unique.as_slice() {
        [] => {
            errors.push(NO_PIN.to_string());
            None
        }
        [pin] => {
            for citation in citations {
                if !same_target(pin, citation) {
                    errors.push(mismatch(pin, citation));
                }
            }
            Some(pin.clone())
        }
        many => {
            errors.push(different_pins(many));
            for citation in citations {
                if !many.iter().any(|pin| same_target(pin, citation)) {
                    errors.push(format!(
                        "{}:{}: {}/{} {} {} is not one of the README pins",
                        citation.path,
                        citation.line,
                        citation.owner,
                        citation.repo,
                        citation.kind.as_str(),
                        citation.sha
                    ));
                }
            }
            None
        }
    }
}

fn unique_pins(pins: &[Pin]) -> Vec<Pin> {
    let mut unique = Vec::new();
    for pin in pins {
        let seen = unique.iter().any(|kept: &Pin| {
            kept.owner == pin.owner && kept.repo == pin.repo && kept.sha == pin.sha
        });
        if !seen {
            unique.push(pin.clone());
        }
    }
    unique
}

fn same_target(pin: &Pin, citation: &Citation) -> bool {
    pin.owner == citation.owner && pin.repo == citation.repo && pin.sha == citation.sha
}

fn mismatch(pin: &Pin, citation: &Citation) -> String {
    format!(
        "{}:{}: {}/{} {} {} is not the README pin {}/{} {}",
        citation.path,
        citation.line,
        citation.owner,
        citation.repo,
        citation.kind.as_str(),
        citation.sha,
        pin.owner,
        pin.repo,
        pin.sha
    )
}

fn different_pins(pins: &[Pin]) -> String {
    let listed = pins
        .iter()
        .map(|pin| {
            format!(
                "README.md:{} {}/{} {}",
                pin.line, pin.owner, pin.repo, pin.sha
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    format!("README.md: more than one commit pin: {listed}")
}

fn pin_label(hit: &Hit) -> Option<&str> {
    let label = hit.link_text.as_deref()?;
    if is_sha_label(label) && label_is_prefix(label, &hit.sha) {
        Some(label)
    } else {
        None
    }
}

struct Hit {
    path: String,
    line: usize,
    owner: String,
    repo: String,
    kind: Kind,
    sha: String,
    sha_ok: bool,
    link_text: Option<String>,
    url: String,
}

struct Parsed {
    owner: String,
    repo: String,
    kind: Kind,
    sha: String,
    sha_ok: bool,
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for path in entries.flatten().map(|entry| entry.path()) {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if path.is_dir() {
                if name != ".git" && name != "target" {
                    stack.push(path);
                }
            } else if path.extension().is_some_and(|ext| ext == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn scan_file(root: &Path, path: &Path, text: &str) -> Vec<Hit> {
    const MARK: &str = "https://github.com/";
    let rel_path = rel(root, path);
    let mut hits = Vec::new();
    let mut from = 0;
    while let Some(rel_index) = text[from..].find(MARK) {
        let start = from + rel_index;
        let url = cut_url(&text[start..]);
        if let Some(parsed) = parse_github_url(url) {
            hits.push(Hit {
                path: rel_path.clone(),
                line: line_of(text, start),
                owner: parsed.owner,
                repo: parsed.repo,
                kind: parsed.kind,
                sha: parsed.sha,
                sha_ok: parsed.sha_ok,
                link_text: link_text(text, start).map(str::to_string),
                url: url.to_string(),
            });
        }
        from = start + MARK.len();
    }
    hits
}

fn cut_url(tail: &str) -> &str {
    let end = tail.find(is_url_end).unwrap_or(tail.len());
    tail[..end].trim_end_matches(is_trailing_punctuation)
}

fn is_url_end(c: char) -> bool {
    c.is_ascii_whitespace() || c == ')' || c == '>' || c == '"' || c == '\'' || c == ']'
}

fn is_trailing_punctuation(c: char) -> bool {
    c == '.' || c == ',' || c == ';' || c == ':'
}

fn parse_github_url(url: &str) -> Option<Parsed> {
    let rest = url.strip_prefix("https://github.com/")?;
    let rest = before_query(rest);
    let mut parts = rest.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    let kind = parse_kind(parts.next()?)?;
    if !is_github_name(owner) || !is_github_name(repo) {
        return None;
    }
    let sha = parts.next().unwrap_or("");
    Some(Parsed {
        owner: owner.to_string(),
        repo: repo.to_string(),
        kind,
        sha_ok: is_full_sha(sha),
        sha: sha.to_string(),
    })
}

fn before_query(url: &str) -> &str {
    let end = url
        .bytes()
        .position(|byte| byte == b'?' || byte == b'#')
        .unwrap_or(url.len());
    &url[..end]
}

fn parse_kind(kind: &str) -> Option<Kind> {
    Some(match kind {
        "commit" => Kind::Commit,
        "blob" => Kind::Blob,
        "tree" => Kind::Tree,
        _ => return None,
    })
}

fn is_github_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'_' | b'-'))
}

fn is_full_sha(sha: &str) -> bool {
    sha.len() == 40 && sha.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

fn is_sha_label(label: &str) -> bool {
    (7..=40).contains(&label.len()) && label.bytes().all(|b| b.is_ascii_hexdigit())
}

fn label_is_prefix(label: &str, sha: &str) -> bool {
    let label = label.as_bytes();
    let sha = sha.as_bytes();
    label.len() <= sha.len()
        && label
            .iter()
            .zip(sha)
            .all(|(label_byte, sha_byte)| label_byte.to_ascii_lowercase() == *sha_byte)
}

fn line_of(text: &str, index: usize) -> usize {
    text[..index].bytes().filter(|byte| *byte == b'\n').count() + 1
}

fn link_text(text: &str, url_start: usize) -> Option<&str> {
    let head = text.get(..url_start)?.strip_suffix("](")?;
    let open = head.rfind('[')?;
    let raw = &head[open + 1..];
    if raw.contains('\n') || raw.contains('\r') {
        return None;
    }
    // The README writes the pin as `[`acc61b3`](url)`, so the text includes backticks.
    let label = match raw
        .trim()
        .strip_prefix('`')
        .and_then(|rest| rest.strip_suffix('`'))
    {
        Some(inner) => inner.trim(),
        None => raw.trim(),
    };
    if label.is_empty() {
        None
    } else {
        Some(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SHA: &str = "acc61b3892ceb6afecfa96c94132bf6e35bbf1fb";
    const OTHER: &str = "0123456789abcdef0123456789abcdef01234567";

    struct Scratch(PathBuf);

    impl Scratch {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "rind-pin-check-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn write(&self, rel_path: &str, text: &str) {
            let path = self.0.join(rel_path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            if self
                .0
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("rind-pin-check-"))
            {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
    }

    fn readme_for(sha: &str) -> String {
        let label = &sha[..7];
        format!("Checked against [`{label}`](https://github.com/kixel-ai/rind/commit/{sha}).\n")
    }

    #[test]
    fn this_repository_has_one_pin() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let report = inspect(&root);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        let pin = report.pin.expect("pin");
        assert_eq!(pin.owner, "kixel-ai");
        assert_eq!(pin.repo, "rind");
        assert_eq!(pin.sha, SHA);
        assert_eq!(pin.label, "acc61b3");
        assert_eq!(pin.line, 11);
        let spots: Vec<_> = report
            .citations
            .iter()
            .map(|citation| (citation.path.as_str(), citation.line, citation.kind))
            .collect();
        assert_eq!(
            spots,
            vec![
                ("README.md", 11, Kind::Commit),
                ("skills/bench/SKILL.md", 18, Kind::Commit),
                ("skills/contribute/SKILL.md", 8, Kind::Blob),
                ("skills/contribute/SKILL.md", 10, Kind::Blob),
                ("skills/crypto-review/SKILL.md", 9, Kind::Blob),
                ("skills/rind/SKILL.md", 20, Kind::Blob),
                ("skills/rind/SKILL.md", 21, Kind::Blob),
                ("skills/rind/SKILL.md", 22, Kind::Blob),
                ("skills/rind/SKILL.md", 23, Kind::Blob),
            ]
        );
    }

    #[test]
    fn accepts_blob_tree_and_ignores_other_urls() {
        let root = Scratch::new();
        root.write("README.md", &readme_for(SHA));
        root.write(
            "skills/rind/SKILL.md",
            &format!(
                "See [AGENTS.md](https://github.com/kixel-ai/rind/blob/{SHA}/AGENTS.md).\n\
                 Tree: <https://github.com/kixel-ai/rind/tree/{SHA}>\n\
                 Bare https://github.com/kixel-ai/rind/blob/{SHA}/FILE.md.\n\
                 Issue: https://github.com/kixel-ai/rind/issues/1\n\
                 History: https://github.com/kixel-ai/rind/commits/{SHA}\n\
                 Home: https://github.com/kixel-ai/rind\n\
                 Digest aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa is not a commit.\n"
            ),
        );
        root.write(
            "target/noise.md",
            &format!("[`0123456`](https://github.com/kixel-ai/rind/commit/{OTHER})\n"),
        );
        root.write(
            ".github/note.md",
            &format!("https://github.com/kixel-ai/rind/commit/{SHA}?diff=split\n"),
        );
        let report = inspect(&root.0);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert_eq!(report.pin.unwrap().label, "acc61b3");
        let spots: Vec<_> = report
            .citations
            .iter()
            .map(|citation| (citation.path.as_str(), citation.kind))
            .collect();
        assert_eq!(
            spots,
            vec![
                (".github/note.md", Kind::Commit),
                ("README.md", Kind::Commit),
                ("skills/rind/SKILL.md", Kind::Blob),
                ("skills/rind/SKILL.md", Kind::Tree),
                ("skills/rind/SKILL.md", Kind::Blob),
            ]
        );
    }

    #[test]
    fn rejects_a_short_sha_a_branch_and_uppercase() {
        let root = Scratch::new();
        root.write("README.md", &readme_for(SHA));
        root.write(
            "skills/rind/SKILL.md",
            "https://github.com/kixel-ai/rind/blob/acc61b3/AGENTS.md\n\
             https://github.com/kixel-ai/rind/blob/main/AGENTS.md\n\
             https://github.com/kixel-ai/rind/commit/ACC61B3892CEB6AFECFA96C94132BF6E35BBF1FB\n",
        );
        let report = inspect(&root.0);
        assert_eq!(report.errors.len(), 3, "{:?}", report.errors);
        assert!(report
            .errors
            .iter()
            .all(|error| error.contains("not a full lowercase commit SHA")));
        assert!(report.pin.is_some());
    }

    #[test]
    fn rejects_a_different_commit_repository_or_label() {
        let root = Scratch::new();
        root.write(
            "README.md",
            &format!(
                "{}[`deadbee`](https://github.com/kixel-ai/rind/commit/{SHA}).\n",
                readme_for(SHA)
            ),
        );
        root.write(
            "skills/bench/SKILL.md",
            &format!(
                "https://github.com/kixel-ai/rind/commit/{OTHER}\n\
                 https://github.com/moq-dev/rind/blob/{SHA}/AGENTS.md\n"
            ),
        );
        let report = inspect(&root.0);
        let pin = report.pin.expect("pin");
        assert_eq!(pin.sha, SHA);
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("link text \"deadbee\" is not a prefix")),
            "{:?}",
            report.errors
        );
        assert!(
            report.errors.iter().any(|error| {
                error.contains(&format!(
                    "kixel-ai/rind commit {OTHER} is not the README pin"
                ))
            }),
            "{:?}",
            report.errors
        );
        assert!(
            report
                .errors
                .iter()
                .any(|error| error
                    .contains(&format!("moq-dev/rind blob {SHA} is not the README pin"))),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn rejects_more_than_one_pin_and_a_sha_outside_them() {
        let root = Scratch::new();
        let third = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
        root.write(
            "README.md",
            &format!(
                "{}\n[`0123456`](https://github.com/kixel-ai/rind/commit/{OTHER})\n",
                readme_for(SHA)
            ),
        );
        root.write(
            "skills/bench/SKILL.md",
            &format!("https://github.com/kixel-ai/rind/tree/{third}\n"),
        );
        let report = inspect(&root.0);
        assert!(report.pin.is_none());
        assert!(
            report.errors.iter().any(|error| {
                error.contains("more than one commit pin")
                    && error.contains(SHA)
                    && error.contains(OTHER)
            }),
            "{:?}",
            report.errors
        );
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("is not one of the README pins")
                    && error.contains(third)),
            "{:?}",
            report.errors
        );
    }

    #[test]
    fn rejects_a_missing_pin_and_unreadable_markdown() {
        let empty = Scratch::new();
        let missing = inspect(&empty.0);
        assert_eq!(missing.errors, vec!["README.md: missing".to_string()]);
        assert!(missing.citations.is_empty());

        let file = empty.0.join("file");
        fs::write(&file, "x").unwrap();
        let not_dir = inspect(&file);
        assert!(
            not_dir
                .errors
                .iter()
                .any(|error| error.contains("not a directory")),
            "{:?}",
            not_dir.errors
        );

        let root = Scratch::new();
        root.write("README.md", "");
        let no_pin = inspect(&root.0);
        assert_eq!(no_pin.errors, vec![NO_PIN.to_string()]);

        root.write("README.md", &readme_for(SHA));
        fs::write(root.0.join("notes.md"), [0xff, 0xfe]).unwrap();
        let bad = inspect(&root.0);
        assert!(bad.pin.is_some());
        assert!(
            bad.errors
                .iter()
                .any(|error| error.starts_with("notes.md:")),
            "{:?}",
            bad.errors
        );
    }

    #[test]
    fn accepts_an_uppercase_label_that_prefixes_the_sha() {
        let root = Scratch::new();
        root.write(
            "README.md",
            &format!("[`ACC61B3`](https://github.com/kixel-ai/rind/commit/{SHA})\n"),
        );
        let report = inspect(&root.0);
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert_eq!(report.pin.unwrap().label, "ACC61B3");
    }
}
