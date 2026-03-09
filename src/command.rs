use std::collections::HashMap;

pub trait TCommand: std::fmt::Debug {
    fn name(&self) -> &str;
    fn execute(&self);
    fn help(&self) -> &str;
    fn version(&self) -> &str;
}
#[derive(Debug)]
pub struct CommandManager {
    commands: HashMap<String, Box<TerminalCommand>>,
}
impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }
    pub fn add_command(&mut self, command: TerminalCommand) {
        self.commands
            .insert(command.run.name().to_string(), Box::new(command));
    }
    pub fn get_command(&self, name: &str) -> Option<&TerminalCommand> {
        self.commands.get(name).map(|c| c.as_ref())
    }
}
#[derive(Debug)]
pub struct TerminalCommand {
    run: Box<dyn TCommand>,
    status: CommandStatus,
}
#[derive(Debug, Clone, PartialEq, Default)]
//Represents all status of a command
enum CommandStatus {
    Installed,
    #[default]
    NotInstalled,
    //Installing with progress
    Installing {
        progress: u8,
    },
    Restarting,
    Failed,
    //Error on action
    Error(String),
    Uninstalling,
}
