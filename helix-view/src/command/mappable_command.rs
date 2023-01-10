use super::{
    Command,
    COMMAND_LIST,
    CommandArgument
};

/// The start of a command that can eventually be mapped to a Command in the COMMAND_LIST.
/// Name of the command name and never a command alias, simplifies partial equality checking.
pub struct MappableCommand {
    name: &str,
    pub supplied_args: &mut [&str]    
}

impl MappableCommand {
    pub fn complete_and_execute(&self, &ui_tree: UITree) {
        todo!()
        // Pseudocode ish for completing arguments
        if Some(uncompleted_argument_index) self.supplied_args.iter().position(|arg| arg == "-") {
            let completeted_argument = await argument_completer(self.supplied_args[uncompleted_argument_index]);
            self.supplied_args[uncompletex_argument_index] = completed_argument;
            self.complete_and_execute(ui_tree);
        }
        else {
            // Calc nr of expected arguments, and check if enough have been supplied (Also psedocode ish)
            // IMPROVEMENT: should need to be calculated once
            let mut required_arguments: usize = 0;
            for argument in COMMAND_LIST.get(self.name).expect("Mappable should only be created if its name exists in COMMAND_MAP.").args {
                match argument {
                    CommandArgument::Required => { required_arguments += 1; },
                    CommandArgument::Optional => {},
                }
            }
            if self.supplied_args < required_arguments {
                ui_tree.error_bar.set("Insufficient arguments provided for {}, expected {}, got {}."
                    ,self.command, required_arguments, self.supplied_args.len());
            }

            COMMAND_MAP.get(self.name).function(ui_tree)
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }
}

impl std::str::FromStr for MappableCommand {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split(' ');
        let name_or_alias = parts.next()
            .ok_or_else(|| anyhow!("Expected a string for the command name (or its alias)."))?;
        let supplied_args = parts.collect();
        if Some (command) = COMMAND_MAP.get(name_or_alias) {
            if args.len() > command.args.len() {
                return Err(anyhow!("Command '{name_or_alias}' expected at most {}, recieved {}", command.args.len(), supplied_args.len()))
            }
            return Ok(MappableCommand { name: command.name, supplied_args })
        }
        else { return Err(anyhow!("No Command named {name}")) }
    }
}

impl<'de> Deserialize<'de> for MappableCommand {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

impl PartialEq for MappableCommand {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}