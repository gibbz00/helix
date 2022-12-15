use std::collections::{HashMap, LinkedList};

#[derive(Debug, Default)]
pub struct Registers {
    registers: HashMap<char, LinkedList<String>>,
}

impl Registers {
    pub fn get(&self, name: char) -> Option<&LinkedList<String>> {
        self.registers.get(&name)
    }

    pub fn write(&mut self, name: char, values: LinkedList<String>) {
        if name != '_' {
            self.registers.insert(name, values);
        }
    }

    pub fn push(&mut self, name: char, value: String) {
        if name != '_' {
            if let Some(register) = self.registers.get_mut(&name) {
                register.push_front(value);
            } else {
                let mut temp_list = LinkedList::new();
                temp_list.push_front(value);
                self.write(name, temp_list);
            }
        }
    }

    pub fn first(&self, name: char) -> Option<&String> {
        self.get(name).and_then(|register| register.back())
    }

    pub fn last(&self, name: char) -> Option<&String> {
        self.get(name).and_then(|register| register.front())
    }

    pub fn get_all(&self) -> &HashMap<char, LinkedList<String>> {
        &self.registers
    }
}
