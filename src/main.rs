use std::env;
use std::fs::{self, DirEntry};
use std::path::Path;

use colored::*;
use regex::Regex;

#[derive(Debug)]
enum FlagType {
    SearchType,
    SearchPath,
    RegexPattern,
    Depth,
}

#[derive(Debug, Default, Clone, Copy)]
enum SearchType {
    #[default]
    Both,
    File,
    Directory,
}

#[derive(Debug, Default)]
struct Cmd {
    pub path: String,
    pub kind: SearchType,
    pub depth: Option<u32>,
    pub pattern: String,
    pub matches: Vec<DirEntry>,
}

impl Cmd {
    pub const PATTERN: &str = r#"\B-(?P<kind>\w+)=(?P<value>[\w./\-]+)"#;

    pub fn new(cmd_str: &str) -> Self {
        let mut instance = Self::default();
        Self::parse_flags(&mut instance, cmd_str);
        if instance.path.is_empty() {
            instance.path = ".".to_string();
        }
        instance
    }

    fn parse_flags(instance: &mut Self, cmd_str: &str) {
        let rgx = Regex::new(Self::PATTERN).unwrap();
        for capt in rgx.captures_iter(cmd_str) {
            let flag_range = capt.name("kind").unwrap().range();
            let val_range = capt.name("value").unwrap().range();

            let flag = cmd_str.get(flag_range).unwrap();
            let value = cmd_str.get(val_range).unwrap();

            if let Some(val) = Self::determine_flag(&flag) {
                match val {
                    FlagType::SearchType => Self::set_kind(instance, value),
                    FlagType::SearchPath => instance.path = value.to_string(),
                    FlagType::RegexPattern => instance.pattern = value.to_string(),
                    FlagType::Depth => instance.depth = Some(value.parse().unwrap()),
                }
            }
        }
    }

    fn determine_flag(flag_str: &str) -> Option<FlagType> {
        match flag_str {
            "t" | "type" => Some(FlagType::SearchType),
            "r" | "regex" => Some(FlagType::RegexPattern),
            "d" | "depth" => Some(FlagType::Depth),
            "p" | "path" => Some(FlagType::SearchPath),
            _ => None,
        }
    }

    fn set_kind(instance: &mut Self, kind_str: &str) {
        instance.kind = match kind_str {
            "b" | "both" => SearchType::Both,
            "f" | "file" => SearchType::File,
            "d" | "directory" => SearchType::Directory,
            _ => SearchType::Both,
        };
    }

    fn print_result(&self) {
        for (i, el) in self.matches.iter().enumerate() {
            let file_name = el.path();
            let file_name = file_name.to_str().unwrap();
            print!("{:4}: ", i + 1);
            if el.file_type().unwrap().is_dir() {
                println!("{}", format!("{}/", file_name).yellow());
            } else {
                println!("{}", file_name.green());
            }
        }
    }
}

fn is_directory(path: &str) -> bool {
    if !Path::new(path).exists() {
        return false;
    }
    let path_metadata = fs::metadata(path).unwrap();
    path_metadata.is_dir()
}

fn abort(msg: &str) {
    eprintln!("{}", msg);
    std::process::exit(1);
}

fn read_dir(path: &str, kind: SearchType, depth: Option<u32>) -> Vec<DirEntry> {
    if !is_directory(path) {
        abort(
            format!(
                "Make sure that the specified filepath is of type directory\n\t{}",
                path
            )
            .as_str(),
        );
    }

    let content = fs::read_dir(path);
    if content.is_err() {
        abort("The specified file path doesn't exist");
    }

    let content = content.unwrap();

    let mut items = Vec::<DirEntry>::new();
    for el in content {
        if let Err(_) = &el {
            abort("Couldn't read directory. Try again!");
        }

        let el = el.unwrap();

        if is_directory(&el.path().to_str().unwrap()) {
            let mut new_depth: Option<u32> = None;
            if depth.is_some() {
                if depth.unwrap() > 0 {
                    new_depth = Some(depth.unwrap() - 1);
                } else {
                    new_depth = Some(0);
                }
            }

            let should_recurse = match new_depth {
                Some(val) => val > 0,
                None => true,
            };

            if should_recurse {
                let mut recursed_items = read_dir(&el.path().to_str().unwrap(), kind, new_depth);

                items.append(&mut recursed_items);
            }
        }

        let matches_kind = match kind {
            SearchType::File => el.file_type().unwrap().is_file(),
            SearchType::Directory => el.file_type().unwrap().is_dir(),
            SearchType::Both => {
                el.file_type().unwrap().is_file() || el.file_type().unwrap().is_dir()
            }
        };

        if matches_kind {
            items.push(el);
        }
    }
    items
}

fn execute_cmd(cmd: &mut Cmd) {
    let items = read_dir(&cmd.path, cmd.kind, cmd.depth);
    // If no items match the requirement, there's no point in trying to find a match
    if items.is_empty() {
        return;
    }

    let rgx = Regex::new(&format!("({})", cmd.pattern));
    if let Err(_) = rgx {
        abort("Sorry, incorrect pattern specified.");
    }

    let rgx = rgx.unwrap();
    for item in items {
        if rgx.is_match(item.file_name().to_str().unwrap()) {
            cmd.matches.push(item);
        }
    }
}

fn main() {
    let mut argv = String::new();
    for arg in env::args() {
        argv.push_str(&arg);
        argv.push(' ');
    }

    let mut cmd = Cmd::new(&argv);
    execute_cmd(&mut cmd);
    cmd.print_result();
}
