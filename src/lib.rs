//! Loads the skills in this repository and checks that they stay usable.
//!
//! The library has no dependencies. It does not run a shell.

use std::fs;
use std::path::{Path, PathBuf};

/// One skill directory: frontmatter plus the Markdown body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub body: String,
}

/// A rind task and the skill that describes it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Task {
    Build,
    Test,
    Bench,
    Review,
    Contribute,
}

impl Task {
    /// `build`, `test`, `bench`, `review`, or `contribute`.
    pub fn parse(name: &str) -> Option<Self> {
        Some(match name {
            "build" => Self::Build,
            "test" => Self::Test,
            "bench" => Self::Bench,
            "review" => Self::Review,
            "contribute" => Self::Contribute,
            _ => return None,
        })
    }

    /// The skill directory for this task.
    pub fn skill_name(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Test => "test",
            Self::Bench => "bench",
            Self::Review => "crypto-review",
            Self::Contribute => "contribute",
        }
    }
}

/// The skills under `root/skills`, sorted by name.
pub fn load(root: &Path) -> Result<Vec<Skill>, Vec<String>> {
    let names = skill_names(root)?;
    let mut skills = Vec::new();
    let mut errors = Vec::new();
    for name in names {
        match read_skill(root, &name) {
            Ok(skill) => skills.push(skill),
            Err(msgs) => errors.extend(msgs),
        }
    }
    if errors.is_empty() {
        Ok(skills)
    } else {
        Err(errors)
    }
}

/// Problems with skill shape, the README table, relative links, and
/// obvious private paths. Empty means the tree is usable.
pub fn check(root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    let readme = fs::read_to_string(root.join("README.md")).unwrap_or_default();
    match skill_names(root) {
        Ok(names) => {
            for name in names {
                match read_skill(root, &name) {
                    Ok(skill) => {
                        if !readme.contains(&format!("](skills/{}/SKILL.md)", skill.name)) {
                            errors.push(format!(
                                "README.md: skill table has no row for {}",
                                skill.name
                            ));
                        }
                    }
                    Err(msgs) => errors.extend(msgs),
                }
            }
        }
        Err(msgs) => errors.extend(msgs),
    }
    for path in markdown_files(root) {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let shown = display_rel(root, &path);
        for (n, line) in text.lines().enumerate() {
            for found in private_line(line) {
                errors.push(format!("{shown}:{}: {found}", n + 1));
            }
        }
        let base = path.parent().unwrap_or(root);
        for target in link_targets(&text) {
            if target.starts_with('#') || has_scheme(target) {
                continue;
            }
            let file = target.split('#').next().unwrap_or(target);
            if !base.join(file).exists() {
                errors.push(format!("{shown}: broken link {target}"));
            }
        }
    }
    errors
}

/// The skill for `task`, if `load` succeeded and the skill is present.
pub fn skill_for(skills: &[Skill], task: Task) -> Option<&Skill> {
    let name = task.skill_name();
    skills.iter().find(|s| s.name == name)
}

fn skill_names(root: &Path) -> Result<Vec<String>, Vec<String>> {
    let dir = root.join("skills");
    let entries = fs::read_dir(&dir).map_err(|e| vec![format!("skills/: {e}")])?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| vec![format!("skills/: {e}")])?;
        if entry.path().is_dir() {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}

fn read_skill(root: &Path, name: &str) -> Result<Skill, Vec<String>> {
    let path = root.join("skills").join(name).join("SKILL.md");
    let text =
        fs::read_to_string(&path).map_err(|_| vec![format!("skills/{name}/SKILL.md: missing")])?;
    let Some((front, body)) = split_frontmatter(&text) else {
        return Err(vec![format!(
            "skills/{name}/SKILL.md: missing YAML frontmatter"
        )]);
    };
    let mut errors = Vec::new();
    if !is_kebab(name) {
        errors.push(format!(
            "skills/{name}/SKILL.md: directory is not lowercase kebab-case"
        ));
    }
    let got = field(front, "name").unwrap_or("");
    if got != name {
        errors.push(format!("skills/{name}/SKILL.md: name must be {name:?}"));
    }
    let description = field(front, "description").unwrap_or("").trim();
    if description.is_empty() {
        errors.push(format!("skills/{name}/SKILL.md: description is empty"));
    } else if !description.starts_with("Use when") {
        errors.push(format!(
            "skills/{name}/SKILL.md: description must start with \"Use when\""
        ));
    }
    if !errors.is_empty() {
        return Err(errors);
    }
    Ok(Skill {
        name: name.to_string(),
        description: description.to_string(),
        body: body.trim().to_string(),
    })
}

fn split_frontmatter(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix("---\n")?;
    let end = rest.find("\n---\n")?;
    Some((&rest[..end], &rest[end + 5..]))
}

fn field<'a>(front: &'a str, key: &str) -> Option<&'a str> {
    front.lines().find_map(|line| {
        let (k, v) = line.split_once(": ")?;
        (k == key).then_some(v)
    })
}

fn is_kebab(s: &str) -> bool {
    !s.is_empty()
        && s.split('-').all(|w| {
            !w.is_empty()
                && w.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        })
}

fn markdown_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(root, &mut out);
    out.sort();
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n == "target") {
                continue;
            }
            walk(&path, out);
        } else if path.extension().is_some_and(|e| e == "md") {
            out.push(path);
        }
    }
}

fn display_rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn link_targets(text: &str) -> impl Iterator<Item = &str> {
    text.match_indices("](").filter_map(move |(i, _)| {
        let rest = &text[i + 2..];
        let end = rest.find(|c: char| c == ')' || c.is_whitespace())?;
        (end > 0 && rest[end..].starts_with(')')).then(|| &rest[..end])
    })
}

fn has_scheme(target: &str) -> bool {
    target
        .split_once(':')
        .is_some_and(|(s, _)| !s.is_empty() && s.bytes().all(|b| b.is_ascii_lowercase()))
}

const HOME_PATHS: [&str; 2] = [concat!("/", "home/"), concat!("/", "Users/")];

fn private_line(line: &str) -> Vec<&'static str> {
    let mut found = Vec::new();
    if HOME_PATHS.iter().any(|p| line.contains(p)) {
        found.push("home directory path");
    }
    found
}

#[cfg(test)]
fn write_tree(root: &Path, files: &[(&str, &str)]) {
    for (rel, text) in files {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "rind-agent-lite-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn this_repository_checks() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let errors = check(&root);
        assert!(errors.is_empty(), "{errors:?}");
        let skills = load(&root).unwrap();
        assert!(skills.iter().any(|s| s.name == "rind"));
        let review = skill_for(&skills, Task::Review).unwrap();
        assert_eq!(review.name, "crypto-review");
        assert!(review.description.starts_with("Use when"));
    }

    #[test]
    fn parses_tasks() {
        assert_eq!(Task::parse("bench"), Some(Task::Bench));
        assert_eq!(Task::parse("review").unwrap().skill_name(), "crypto-review");
        assert_eq!(Task::parse("other"), None);
    }

    #[test]
    fn rejects_a_broken_skill() {
        let root = scratch();
        write_tree(
            &root,
            &[
                ("README.md", "# x\n\n[build](skills/build/SKILL.md)\n"),
                (
                    "skills/build/SKILL.md",
                    "---\nname: other\ndescription: Helps.\n---\n# Build\n",
                ),
            ],
        );
        let errors = check(&root);
        assert!(
            errors.iter().any(|e| e.contains("name must be")),
            "{errors:?}"
        );
        assert!(errors.iter().any(|e| e.contains("Use when")), "{errors:?}");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn rejects_a_broken_link_and_a_home_path() {
        let root = scratch();
        let home = concat!("see /", "home/someone/rind");
        write_tree(
            &root,
            &[
                ("README.md", "# x\n\n[build](skills/build/SKILL.md)\n"),
                (
                    "skills/build/SKILL.md",
                    &format!(
                        "---\nname: build\ndescription: Use when building.\n---\n\n[missing](nope.md)\n{home}\n"
                    ),
                ),
            ],
        );
        let errors = check(&root);
        assert!(
            errors.iter().any(|e| e.contains("broken link")),
            "{errors:?}"
        );
        assert!(
            errors.iter().any(|e| e.contains("home directory path")),
            "{errors:?}"
        );
        let _ = fs::remove_dir_all(&root);
    }
}
