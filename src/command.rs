use parking_lot::RwLock;
use std::{collections::HashMap, error::Error, sync::Arc};
pub trait Manager<T> {
    fn add(&mut self, generic: T) -> Result<(), impl std::error::Error>;
    fn get(&self, name: &str) -> Option<T>;
    fn remove(&mut self, name: &str);
    fn update(&mut self, name: &str);
}
pub trait PluginError: Error + Send + Sync + 'static {
    fn source(&self) -> Option<&(dyn Error + 'static)>;
    fn backtrace(&self) -> Option<&std::backtrace::Backtrace>;
}
pub trait PluginCommand: std::fmt::Debug + PluginError {
    fn name(&self) -> &str;
    fn execute(&self, args: Vec<String>) -> PluginResult;
    fn help(&self) -> &str;
    fn version(&self) -> &str;
}
#[derive(Debug, Clone)]
pub struct CommandManager {
    commands: HashMap<String, TerminalCommand>,
}

#[derive(Debug, Clone)]
pub struct TerminalCommand {
    run: Arc<dyn PluginCommand>,
    status: CommandStatus,
}
impl TerminalCommand {
    pub fn new(run: Arc<dyn PluginCommand>) -> Self {
        Self {
            run,
            status: CommandStatus::default(),
        }
    }
    pub fn status(&self) -> &CommandStatus {
        &self.status
    }
    pub fn run(&self, args: Vec<String>) -> PluginResult {
        if self.status != CommandStatus::Installed {
            PluginResult::Failed {
                result: 0,
                message: "Command not installed".to_string(),
            }
        } else {
            self.run.execute(args)
        }
    }
    fn set_status(&mut self, status: CommandStatus) -> CommandStatus {
        let old_status = self.status.clone();
        match status {
            CommandStatus::Installing { .. } => {
                self.status = status;
            }
            _ => {
                self.status = status;
            }
        }
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
            commands: HashMap::new(),
        }
    }
}
impl Manager<TerminalCommand> for CommandManager {
    fn add(&mut self, generic: TerminalCommand) -> Result<(), impl Error> {
        let name = generic.run.name().to_string();

        self.commands.insert(name, generic);
        Err(PluginResult::default())
    }
    fn get(&self, name: &str) -> Option<TerminalCommand> {
        self.commands.get(name).cloned()
    }
    fn remove(&mut self, name: &str) {
        match self.commands.get_mut(name) {
            Some(value) => {
                _ = value.remove();
            }
            None => {}
        }
    }
    fn update(&mut self, name: &str) {
        match self.commands.get_mut(name) {
            Some(value) => {
                value.check();
            }
            None => {}
        }
    }
}
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PluginResult {
    #[default]
    Success,
    Failed {
        result: isize,
        message: String,
    },
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
impl std::fmt::Display for PluginResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PluginResult::Success => write!(f, "Success"),
            PluginResult::Failed { message, .. } => write!(f, "Failed: {}", message),
        }
    }
}
impl std::error::Error for PluginResult {}
impl PluginError for PluginResult {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
    fn backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        None
    }
}
