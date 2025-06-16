use std::{fmt, io};

use crate::objects::{read_object, ObjectType, Tree, TreeEntry};


pub fn run(args: &[String]) -> io::Result<()> {
    let cmd = parse_command(args)?;
    let (object_type, _) = read_object(&cmd.tree_ish())?;
    if object_type != ObjectType::Tree {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("Object {} is not a tree", cmd.tree_ish())));
    }

    let tree = Tree::load_tree_from_hash(&cmd.tree_ish())?;
        tree.walk_tree(cmd.base_path(), &mut |entry, path| {
        cmd.printer().print(entry, path);
    },cmd.recursive())?;

    Ok(())
}

pub fn parse_command(args: &[String]) -> io::Result<LsTreeCommand> {
    let mut tree_ish: Option<String> = None;
    let mut printer: Option<Box<dyn TreeEntryPrinter>> = None;

    for arg in args {
        match arg.as_str() {
            "--name-only" => {
                printer = Some(Box::new(NameOnlyPrinter));
            }
            x if x.starts_with('-') => {
                return Err(CommandParseError::UnknownFlag(x.to_string()).into());
            }
            other => {
                if let Some(already) = tree_ish.replace(other.to_string()) {
                    return Err(CommandParseError::InvalidArgument(format!(
                        "Multiple tree-ish values: '{}' and '{}'", already, other
                    )).into());
                }
            }
        }
    }

    let tree_ish = tree_ish.ok_or_else(|| {
        CommandParseError::MissingArgument("tree-ish (e.g. a hash or HEAD)".to_string())
    })?;

    let printer = printer.unwrap_or_default();

    Ok(LsTreeCommand {
        tree_ish,
        base_path: "".to_string(),
        printer,
        recursive: false,
    })
}

pub struct LsTreeCommand {
    tree_ish: String,
    base_path: String,
    printer: Box<dyn TreeEntryPrinter>,
    recursive: bool,
}

impl LsTreeCommand {
        pub fn tree_ish(&self) -> &str {
        &self.tree_ish
    }

    pub fn base_path(&self) -> &str {
        &self.base_path
    }

    pub fn printer(&self) -> &dyn TreeEntryPrinter {
        self.printer.as_ref()
    }

    pub fn recursive(&self) -> bool {
        self.recursive
    }
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

pub trait  TreeEntryPrinter {
    fn print(&self, entry: &TreeEntry, path: &str);
}

pub struct NameOnlyPrinter;

impl TreeEntryPrinter for NameOnlyPrinter {
    fn print(&self, _entry: &TreeEntry, path: &str) {
        println!("{}", path);
    }
}

pub struct DefaultPrinter;

impl TreeEntryPrinter for DefaultPrinter {
    fn print(&self, entry: &TreeEntry, path: &str) {
        println!(
            "{} {} {}\t{}",
            entry.mode,
            entry.object_type,
            entry.hash.to_hex(),
            path
        );
    }
}

impl Default for Box<dyn TreeEntryPrinter> {
    fn default() -> Self {
        Box::new(DefaultPrinter)
    }
}