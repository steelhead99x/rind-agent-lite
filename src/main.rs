//! Command line for the commit-pin check.

#![forbid(unsafe_code)]

use std::env;
use std::path::PathBuf;
use std::process::ExitCode;

use rind_pin_check::inspect;

const USAGE: &str = "\
usage: rind-pin-check [DIR]
       rind-pin-check list [DIR]";

enum Args {
    Help,
    Run { list: bool, dir: Option<String> },
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let args = match parse(&args) {
        Ok(args) => args,
        Err(()) => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let Args::Run { list, dir } = args else {
        println!("{USAGE}");
        return ExitCode::SUCCESS;
    };
    let root = match dir {
        Some(dir) => PathBuf::from(dir),
        None => match find_root() {
            Some(root) => root,
            None => {
                eprintln!("run rind-pin-check from the repository (README.md and skills/)");
                return ExitCode::from(2);
            }
        },
    };
    let report = inspect(&root);
    if list {
        for citation in &report.citations {
            println!(
                "{}:{} {} {}/{} {}",
                citation.path,
                citation.line,
                citation.kind.as_str(),
                citation.owner,
                citation.repo,
                citation.sha
            );
        }
    }
    for error in &report.errors {
        eprintln!("{error}");
    }
    if !report.errors.is_empty() {
        return ExitCode::FAILURE;
    }
    if let Some(pin) = &report.pin {
        println!("pin {} {} {}/{}", pin.sha, pin.label, pin.owner, pin.repo);
    }
    ExitCode::SUCCESS
}

fn parse(args: &[String]) -> Result<Args, ()> {
    match args {
        [] => Ok(Args::Run {
            list: false,
            dir: None,
        }),
        [arg] if arg == "--help" || arg == "-h" => Ok(Args::Help),
        [arg] if arg == "list" => Ok(Args::Run {
            list: true,
            dir: None,
        }),
        [dir] => Ok(Args::Run {
            list: false,
            dir: Some(dir.clone()),
        }),
        [cmd, dir] if cmd == "list" => Ok(Args::Run {
            list: true,
            dir: Some(dir.clone()),
        }),
        _ => Err(()),
    }
}

fn find_root() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        if dir.join("README.md").is_file() && dir.join("skills").is_dir() {
            return Some(dir);
        }
        if !dir.pop() {
            return None;
        }
    }
}
