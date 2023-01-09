mod command_list;
mod mappable_command;
mod client;

pub use command_list::COMMAND_LIST as COMMAND_LIST;
pub use command_list::COMMAND_MAP as COMMAND_MAP;
use crate::lists::List;

#[derive(Clone, Debug)]
pub struct Command {
    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
    args: &'static [&'static CommandArgument],
    function: fn(&self)
}

#[derive(FromStr)]
pub enum CommandArgument {
    Required(CommandArgumentVariants),
    Optional(CommandArgumentVariants),
}

pub enum CommandArgumentVariants {
    FilePath,
    FilePaths(Vec<Path>),
    DicectoryPath,
    Buffer,
    Buffers(Vec<Buffer>),
    IndentStyle((Option<Spacing>, usize)),
    LineEnding(LineEnding),
    UndoKind(helix_core::history::UndoKind),
    Theme,
    Languages,
    ConfigOptions,
    ShellCommand,
    Char(char),
    Integer(isize)
}


enum Spacing {
    Tabs = "t",
    Spaces = "s",
}

#[cfg(not(feature = "unicode-lines"))]
enum LineEnding {
    CRLF = "crlf",
    LF = "lf",
}
#[cfg(feature = "unicode-lines")]
enum LineEnding {
    CRLF = "crlf",
    LF = "lf",
    CR = "cr",
    FF = "ff",
    NEL = "nel"
}
