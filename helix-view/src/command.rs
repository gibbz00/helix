mod command_list;
mod mappable_command;

pub use command_list::COMMAND_LIST as COMMAND_LIST;
use crate::lists::List;

#[derive(Clone, Debug)]
pub struct Command {
    name: &'static str,
    aliases: &'static [&'static str],
    description: &'static str,
    args: &'static [&'static CommandArguments],
}

#[derive(FromStr)]
pub enum CommandArguments {
    FilePath,
    OptionalFilePath(Option<Path>),
    FilePaths(Vec<Path>),
    OptionalFilePaths(Option<Paths>),
    DicectoryPath,
    Buffer,
    Buffers(Vec<Buffer>),
    OptionalBuffers(Option<Buffers>),
    IndentStyle((Option<Spacing>, usize)),
    LineEnding(LineEnding),
    OptionalUndoKind(Option<helix_core::history::UndoKind>),
    OptionalTheme(Option<todo!()>),
    Languages,
    ConfigOptions,
    ShellCommand(&str)
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
