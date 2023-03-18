use std::env;
use std::fs::{self, DirEntry};

use regex::Regex;

fn is_directory(path: &str) -> bool {
    let path_metadata = fs::metadata(path).unwrap();
    path_metadata.is_dir()
}

fn list_files(path: &str, depth: i32) -> Vec<DirEntry> {
    if !is_directory(path) {
        panic!("Make sure that the specified filepath is of type directory");
    }

    let path = fs::read_dir(path);
    if path.is_err() {
        panic!("The specified file path doesn't exist");
    }

    let path = path.unwrap();

    let mut files = Vec::<DirEntry>::new();
    for item in path {
        if let Err(err) = item {
            panic!("{}", err);
        }
        let item = item.unwrap();
        if depth > 0 && item.metadata().unwrap().is_dir() {
            let mut recursed_files = list_files(item.path().to_str().unwrap(), depth - 1);
            files.append(&mut recursed_files);
        } else if item.metadata().unwrap().is_file() {
            files.push(item);
        }
    }

    files
}

#[derive(Debug, Default)]
struct Command {
    dir: String,
    pub kind: SearchType,
    pub depth: Option<u32>,
    pub pattern: String,
}

impl Command {
    pub fn new(cmd_str: &str) -> Self {
        let mut instance = Self::default();
        Self::parse_cmd(&mut instance, cmd_str);
        instance
    }

    fn parse_cmd(instance: &mut Self, cmd_str: &str) {
        let rgx = Regex::new(PATTERN).unwrap();
        for capt in rgx.captures_iter(cmd_str) {
            let flag_range = capt.name("kind").unwrap().range();
            let val_range = capt.name("value").unwrap().range();

            let flag = cmd_str.get(flag_range).unwrap();
            let value = cmd_str.get(val_range).unwrap();

            let mut flag = flag.to_string();
            // Remove the first character which is the hyphen('-')
            // Ex: -t OR -d OR -r
            flag.remove(0);

            if let Some(val) = Self::determine_flag(&flag) {
                match val {
                    FlagType::SearchType => Self::set_kind(instance, value),
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
}

#[derive(Debug)]
enum FlagType {
    SearchType,
    RegexPattern,
    Depth,
}

#[derive(Debug, Default)]
enum SearchType {
    #[default]
    Both,
    File,
    Directory,
}

pub const PATTERN: &str = r#"\B(?P<kind>-\w+) (?P<value>[\w./\-_]+)"#;

fn main() {
    let mut argv = String::new();
    for arg in env::args() {
        argv.push_str(&arg);
        argv.push(' ');
    }

    let cmd = Command::new(&argv);
}
