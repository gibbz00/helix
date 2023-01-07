use super::{
    Command,
    COMMAND_LIST
};
/// The start of a command that can eventually be mapped to a Command in the COMMAND_LIST
pub struct MappableCommand {
    name: &str,
    pub supplied_args: &[&str]    
}

impl MappableCommand {
    /// Promt input of remaining args goes here
    pub fn execute(&self) {
        todo!()
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
            .ok_or_else(|| anyhow!("Expected a string for the command name."))?;
        let supplied_args = parts.collect();
        if Some (command) = COMMAND_LIST.iter().find(|cmd| {
            cmd.name == name_or_alias ||
            cmd.aliases.contains(&name_or_alias)
        }) {
            if args.len() > command.max_expected_args {
                return Err(anyhow!("Command '{name_or_alias}' expected at most {}, recieved {}", command.max_expected_args, supplied_args.len()))
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