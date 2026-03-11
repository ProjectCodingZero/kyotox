use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
pub trait Manager<T> {
    fn add(&mut self, generic: T);
    fn get(&self, name: &str) -> Option<T>;
    fn remove(&mut self, name: &str);
    fn update(&mut self, name: &str);
}
pub trait TCommand: std::fmt::Debug {
    fn name(&self) -> &str;
    fn execute(&self, args: Vec<String>) -> CommandResult;
    fn help(&self) -> &str;
    fn version(&self) -> &str;
}
#[derive(Debug, Clone)]
pub struct CommandManager {
    commands: Arc<RwLock<HashMap<String, TerminalCommand>>>,
}

#[derive(Debug, Clone)]
pub struct TerminalCommand {
    run: Arc<dyn TCommand>,
    status: CommandStatus,
}
impl TerminalCommand {
    pub fn new(run: Arc<dyn TCommand>) -> Self {
        Self {
            run,
            status: CommandStatus::default(),
        }
    }
    pub fn status(&self) -> &CommandStatus {
        &self.status
    }
    pub fn run(&self, args: Vec<String>) -> CommandResult {
        if self.status != CommandStatus::Installed {
            CommandResult::Error("Command not installed".to_string())
        } else {
            self.run.execute(args)
        }
    }
    fn set_status(&mut self, status: CommandStatus) -> CommandStatus {
        let old_status = self.status.clone();
        self.status = status;
        old_status
    }
    #[inline]
    pub fn remove(&mut self) -> CommandStatus {
        self.set_status(CommandStatus::Uninstalling { progress: 0 })
    }
    pub fn install(&mut self) -> CommandStatus {
        self.set_status(CommandStatus::Installing { progress: 0 })
    }
    pub fn restart(&mut self) -> CommandStatus {
        self.set_status(CommandStatus::Restarting)
    }
    pub fn check(&mut self) -> CommandStatus {
        self.set_status(CommandStatus::Installed)
    }
}
impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
impl Manager<TerminalCommand> for CommandManager {
    fn add(&mut self, generic: TerminalCommand) {
        let name = generic.run.name().to_string();
        //wait to get lock
        let mut lock = self.commands.write();
        lock.insert(name, generic);
    }
    fn get(&self, name: &str) -> Option<TerminalCommand> {
        let lock = self.commands.read();
        lock.get(name).cloned()
    }
    fn remove(&mut self, name: &str) {
        let mut lock = self.commands.write();
        match lock.get_mut(name) {
            Some(value) => {
                _ = value.remove();
            }
            None => {}
        }
    }
    fn update(&mut self, name: &str) {
        let mut lock = self.commands.write();
        match lock.get_mut(name) {
            Some(value) => {
                value.check();
            }
            None => {}
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CommandResult {
    #[default]
    Success,
    Failed(usize),
    Error(String),
}
#[derive(Debug, Clone, PartialEq, Default)]
//Represents all status of a command
pub enum CommandStatus {
    Installed,
    #[default]
    NotInstalled,
    //Installing with progress
    Installing {
        progress: u8,
    },
    Restarting,
    Failed,
    Uninstalling {
        progress: u8,
    },
}
