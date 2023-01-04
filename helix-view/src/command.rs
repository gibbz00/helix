enum Consumer {
    Client,
    Server,    
}

#[derive(Clone)]
pub struct Command {
    name: &'static str,
    description: &'static str,
    args: Vec<String>,
    consumer: Consumer
}
