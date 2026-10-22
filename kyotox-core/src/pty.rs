use portable_pty::{CommandBuilder, MasterPty, Child};
pub use portable_pty::PtySize;
use std::{fmt::Debug, io};
pub struct PtySession {
    master: Box<dyn MasterPty + Send>,
    child: Box<dyn Child + Send>,
}
// TODO: Handle errors properly instead of using String
/// Represents errors that can occur during PTY operations.
#[derive(Debug)]
pub enum PtyError {
    SpawnError(String),
    ResizeError(String),
    CommandError(String),
}

impl PtySession {
    pub fn spawn(command: CommandBuilder, configs: PtySize) -> Result<Self, PtyError> {
        log::info!("A new pty session has spawned");
        let pty_system = portable_pty::native_pty_system();
        match pty_system.openpty(configs) {
            Ok(pair) => {
                let child = pair
                    .slave
                    .spawn_command(command)
                    .map_err(|e| PtyError::SpawnError(e.to_string()))?;
                drop(pair.slave); // Close the slave end in the parent process
                Ok(Self {
                    master: pair.master,
                    child,
                })
            }
            Err(e) => {
                //TODO: Handle errors properly instead of using expect
                return Err(PtyError::SpawnError(e.to_string()));
            }
        }
    }

    pub fn resize(&mut self, new_size: PtySize) {
        self.master.resize(new_size).expect("Failed to resize PTY");
    }
    pub fn child_pid(&self) -> Option<u32> {
        self.child.process_id()
    }
    pub fn kill_child(&mut self) -> Result<(), PtyError> {
        if let Some(pid) = self.child.process_id() {
            // Attempt to terminate the child process gracefully
            if let Err(e) = self.child.kill() {
                eprintln!("Failed to kill child process {}: {}", pid, e);
                let error_message = format!("Failed to kill child process {}: {}", pid, e);
                return Err(PtyError::CommandError(error_message));
            }
        }
        Ok(())
    }
}
impl io::Write for PtySession {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.master.take_writer() {
            Ok(mut writer) => writer.write(buf),
            Err(e) => {
                // TODO: Handle the error by returning an io::Error with a descriptive message
                let err = io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to take writer: {}", e),
                );
                Err(err)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.master.take_writer() {
            Ok(mut writer) => writer.flush(),
            Err(e) => {
                // TODO: Handle the error by returning an io::Error with a descriptive message
                let err = io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to take writer: {}", e),
                );
                Err(err)
            }
        }
    }
}
impl io::Read for PtySession {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.master.try_clone_reader() {
            Ok(mut reader) => reader.read(buf),
            Err(e) => {
                // TODO: Handle the error by returning an io::Error with a descriptive message
                let err = io::Error::new(
                    io::ErrorKind::Other,
                    format!("Failed to take reader: {}", e),
                );
                Err(err)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use std::io::Read;

    use super::*;
    #[test]
    fn pty_spawn() {
        let pty_size = PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        };
        let text = "Hello, PTY!";
        let echo_text = format!("echo '{}'", text);
        let mut cmd = CommandBuilder::new("bash");
        cmd.args(&["-c", echo_text.as_str()]);
        let mut pty_session = PtySession::spawn(cmd, pty_size).unwrap();

        let buffer = &mut [0u8; 1024];
        let reader = pty_session.read(buffer.as_mut()).unwrap();

        let expected_text = format!("{}\r\n", text).into_bytes();
        let actual_text = &buffer[..reader];
        assert_eq!(
            actual_text,
            expected_text
        );
    }
}
