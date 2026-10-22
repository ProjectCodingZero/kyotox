use std::io::Error;
use crate::ky_async::Sender;
use thiserror::Error;
use vte::{Params, Perform};
use crate::grid::cell::Color::Ascii;
use crate::term::{EventTerminal, Terminal};
#[derive(Error, Debug)]
pub enum VteError {
    #[error("Send: {0}")]
    SendError(#[from] smol::channel::SendError<EventTerminal>),
}
#[derive(Debug)]
pub struct VteBridge{
    sender: Sender<EventTerminal>,
    errors: Vec<VteError>
}
impl VteBridge {
    pub fn new(sender: Sender<EventTerminal>) -> Self {
        VteBridge{
            sender,
            errors: Vec::new()
        }
    }
    fn call_once<F>(&mut self, func: F)
    where F: FnOnce() -> Result<(), VteError> {
        if let Err(err) = func() {
            self.errors.push(err)
        }
    }
    pub fn pop_error(&mut self) -> Option<VteError> {
        self.errors.pop()
    }
    pub fn get_errors(&mut self) -> Vec<VteError> {
        std::mem::take(&mut self.errors)
    }
}
impl Perform for VteBridge {
    fn print(&mut self, c: char) {
        let sender = self.sender.clone();
        self.call_once(move || {
            sender
                .send_blocking(EventTerminal::PrintChar(c))
                .map_err(VteError::SendError)
        });
    }
    fn execute(&mut self, byte: u8) {
        if byte == b'\n' {
            let sender = self.sender.clone();

            self.call_once(move || {
                sender
                    .send_blocking(EventTerminal::NewLine)
                    .map_err(VteError::SendError)
            });
        }
    }
    fn csi_dispatch(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, action: char) {
        let message: EventTerminal;
        if action == 'm' {
            // Extraer color y enviar al canal
            message = EventTerminal::ChangeColor(Ascii(31));
            let _ = self.sender.send_blocking(EventTerminal::ChangeColor(Ascii(31))); // Ejemplo fijo
        } else if action == 'J' {
            message = EventTerminal::ClearScreen;
        } else {
            message = EventTerminal::None;
        }
        let sender = self.sender.clone();
        self.call_once(|| {
            sender
                .send_blocking(message)
                .map_err(VteError::SendError)
        })
    }
}
mod test{
    use super::*;
    use vte::Parser;
    use smol::channel::unbounded;
    #[test]
    fn test_parser() {
        let mut parser = Parser::new();
        let (sender, receiver) = unbounded::<EventTerminal>();
        let mut vte = VteBridge::new(sender);
        let input_data = b"A";
        parser.advance(&mut vte, input_data);
        let event = receiver.recv_blocking().unwrap();
        assert_eq!(event, EventTerminal::PrintChar('A'));
    }
}