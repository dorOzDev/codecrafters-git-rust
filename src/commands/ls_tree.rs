use std::{fmt, io};

use crate::{objects::{read_object, ObjectType, Tree}};


pub fn run(args: &[String]) -> io::Result<()> {
    let cmd = parse_command(args)?;
    let (object_type, _) = read_object(&cmd.tree_ish)?;
    if object_type != ObjectType::Tree {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Object {} is not a tree", cmd.tree_ish)));
    }

    let tree = Tree::load_tree_from_hash(&cmd.tree_ish)?;
        tree.walk_tree("", &mut |entry, path| {
        if cmd.options.name_only {
            println!("{}", path);
        } else {
            println!(
                "{} {} {}\t{}",
                entry.mode,
                entry.object_type,
                entry.hash,
                path
            );
        }
    } ,false)?;
    Ok(())
}

fn parse_command(args: &[String]) -> io::Result<LsTreeCommand> {
    let mut tree_ish: Option<String> = None;
    let mut name_only = false;
    for arg in args {
        match arg.as_str() {
            "--name-only" => name_only = true,
            x if x.starts_with('-') => {
                return Err(CommandParseError::UnknownFlag(x.to_string()).into());
            }
            other => {
                if tree_ish.is_some() {
                    return Err(CommandParseError::InvalidArgument(other.to_string()).into());
                }
                tree_ish = Some(other.to_string());
            }
        }
    }

    let tree_ish = tree_ish.ok_or_else(|| {
        CommandParseError::MissingArgument("tree-ish (e.g. a hash or HEAD)".to_string())
    })?;

    Ok(LsTreeCommand {
        options: LsTreeOptions {
            name_only,
        },
        tree_ish: tree_ish,
    })
}

pub struct LsTreeOptions {
    pub name_only: bool,
}

pub struct LsTreeCommand {
    pub options: LsTreeOptions,
    pub tree_ish: String,
}

#[derive(Debug)]
pub enum CommandParseError {
    MissingArgument(String),
    UnknownFlag(String),
    InvalidArgument(String),
}

impl fmt::Display for CommandParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandParseError::MissingArgument(msg) => write!(f, "Missing argument: {}", msg),
            CommandParseError::UnknownFlag(flag) => write!(f, "Unknown flag: {}", flag),
            CommandParseError::InvalidArgument(arg) => write!(f, "Invalid argument: {}", arg),
        }
    }
}

impl std::error::Error for CommandParseError {}

impl From<CommandParseError> for io::Error {
    fn from(err: CommandParseError) -> Self {
        io::Error::new(io::ErrorKind::InvalidInput, err)
    }
}