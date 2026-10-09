//! Command management system and plugin lifecycle traits.
//!
//! Provides abstract interfaces for managing commands, error handling for plugins,
//! and tracking command status and execution results.

// use parking_lot::RwLock;
// use smol::Async;
use std::{collections::HashMap, error::Error, sync::Arc};

/// Generic trait for managing entity collections (such as commands).
pub trait Manager<T> {
    /// Adds an entity to the manager.
    fn add(&mut self, generic: T) -> Result<(), impl std::error::Error>;

    /// Retrieves an entity by its name.
    fn get(&self, name: &str) -> Option<T>;

    /// Removes an entity by name.
    fn remove(&mut self, name: &str);

    /// Updates an entity by name.
    fn update(&mut self, name: &str);
}

/// Trait extending [`std::error::Error`] for plugin errors with backtrace support.
pub trait PluginError: Error + Send + Sync + 'static {
    /// Returns the optional underlying error source.
    fn source(&self) -> Option<&(dyn Error + 'static)>;

    /// Returns an optional error backtrace.
    fn backtrace(&self) -> Option<&std::backtrace::Backtrace>;
}

/// Interface for executable plugin commands.
pub trait PluginCommand: std::fmt::Debug + PluginError {
    /// Returns the unique name of the plugin command.
    fn name(&self) -> &str;

    /// Executes the plugin command with the provided string arguments.
    fn execute(&self, args: Vec<String>) -> PluginResult;

    /// Returns help documentation for the command.
    fn help(&self) -> &str;

    /// Returns the version string of the command.
    fn version(&self) -> &str;
}

/// Holds a registry of terminal commands mapped by their unique names.
#[derive(Debug, Clone)]
pub struct CommandManager {
    commands: HashMap<String, TerminalCommand>,
}

/// Represents a single registered terminal command along with its installation status.
#[derive(Debug, Clone)]
pub struct TerminalCommand {
    run: Arc<dyn PluginCommand>,
    status: CommandStatus,
}

impl TerminalCommand {
    /// Creates a new [`TerminalCommand`] wrapping a plugin command.
    pub fn new(run: Arc<dyn PluginCommand>) -> Self {
        Self {
            run,
            status: CommandStatus::default(),
        }
    }

    /// Returns a reference to the command's current status.
    pub fn status(&self) -> &CommandStatus {
        &self.status
    }

    /// Runs the command if it is installed, returning a [`PluginResult`].
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

    /// Sets the command status and returns the previous status.
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

    /// Marks the command as uninstalling.
    #[inline]
    pub fn remove(&mut self) -> CommandStatus {
        self.set_status(CommandStatus::Uninstalling { progress: 0 })
    }

    /// Marks the command as installing.
    pub fn install(&mut self) -> CommandStatus {
        self.set_status(CommandStatus::Installing { progress: 0 })
    }

    /// Marks the command as restarting.
    pub fn restart(&mut self) -> CommandStatus {
        self.set_status(CommandStatus::Restarting)
    }

    /// Verifies and marks the command as installed.
    pub fn check(&mut self) -> CommandStatus {
        self.set_status(CommandStatus::Installed)
    }
}

impl CommandManager {
    /// Constructs a new, empty [`CommandManager`].
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

/// Result outcome of executing a plugin command.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PluginResult {
    /// Execution completed successfully.
    #[default]
    Success,
    /// Execution failed with an exit result code and message.
    Failed {
        /// Exit code or status result identifier.
        result: isize,
        /// Descriptive failure error message.
        message: String,
    },
}

/// Represents all potential lifecycle statuses of a command.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum CommandStatus {
    /// Command is installed and ready for execution.
    Installed,
    /// Command is not installed.
    #[default]
    NotInstalled,
    /// Command is currently installing, with a progress percentage (0-100).
    Installing {
        /// Progress percentage value.
        progress: u8,
    },
    /// Command is restarting.
    Restarting,
    /// Command installation or execution failed.
    Failed,
    /// Command is currently being uninstalled, with progress.
    Uninstalling {
        /// Progress percentage value.
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
