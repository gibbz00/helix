use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Registers {
    registers: HashMap<char, Vec<String>>,
}

impl Registers {
    pub fn get(&self, name: char) -> Option<&Vec<String>> {
        self.registers.get(&name)
    }

    pub fn write(&mut self, name: char, values: Vec<String>) {
        if name != '_' {
            self.registers.insert(name, values);
        }
    }

    pub fn push(&mut self, name: char, value: String) {
        if name != '_' {
            if let Some(register) = self.registers.get_mut(&name) {
                register.push(value);
            } else {
                self.write(name, vec![value]);
            }
        }
    }

    pub fn first(&self, name: char) -> Option<&String> {
        self.get(name).and_then(|entries| entries.first())
    }

    pub fn last(&self, name: char) -> Option<&String> {
        self.get(name).and_then(|entries| entries.last())
    }

    pub fn inner(&self) -> &HashMap<char, Vec<String>> {
        &self.registers
    }
}
