enum Consumer {
    Client,
    Server,    
}

pub type Commands = Vec<Command>;

#[derive(Clone)]
pub struct Command {
    name: &'static str,
    description: &'static str,
    args: Vec<String>,
    consumer: Consumer
}
