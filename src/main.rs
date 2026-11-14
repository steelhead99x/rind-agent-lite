//! Command line for the skill library.

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use rind_agent_lite::{check, load, skill_for, Task};

const USAGE: &str = "\
usage: rind-agent-lite check
       rind-agent-lite skills
       rind-agent-lite show <skill>
       rind-agent-lite task <build|test|bench|review|contribute>";

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(cmd) = args.next() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let root = find_root();
    match cmd.as_str() {
        "check" => {
            let errors = check(&root);
            for error in &errors {
                eprintln!("{error}");
            }
            if errors.is_empty() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        "skills" => match load(&root) {
            Ok(skills) => {
                for skill in skills {
                    println!("{}: {}", skill.name, skill.description);
                }
                ExitCode::SUCCESS
            }
            Err(errors) => {
                for error in errors {
                    eprintln!("{error}");
                }
                ExitCode::FAILURE
            }
        },
        "show" => {
            let Some(name) = args.next() else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            match load(&root) {
                Ok(skills) => match skills.iter().find(|s| s.name == name) {
                    Some(skill) => print_skill(skill),
                    None => {
                        eprintln!("no skill {name}");
                        ExitCode::FAILURE
                    }
                },
                Err(errors) => report(errors),
            }
        }
        "task" => {
            let Some(name) = args.next() else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            let Some(task) = Task::parse(&name) else {
                eprintln!("{USAGE}");
                return ExitCode::from(2);
            };
            match load(&root) {
                Ok(skills) => match skill_for(&skills, task) {
                    Some(skill) => print_skill(skill),
                    None => {
                        eprintln!("no skill {}", task.skill_name());
                        ExitCode::FAILURE
                    }
                },
                Err(errors) => report(errors),
            }
        }
        _ => {
            eprintln!("{USAGE}");
            ExitCode::from(2)
        }
    }
}

fn print_skill(skill: &rind_agent_lite::Skill) -> ExitCode {
    println!("{}\n\n{}", skill.description, skill.body);
    ExitCode::SUCCESS
}

fn report(errors: Vec<String>) -> ExitCode {
    for error in errors {
        eprintln!("{error}");
    }
    ExitCode::FAILURE
}

fn find_root() -> PathBuf {
    let mut dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    loop {
        if dir.join("skills").is_dir() && dir.join("README.md").is_file() {
            return dir;
        }
        if !dir.pop() {
            return env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        }
    }
}
