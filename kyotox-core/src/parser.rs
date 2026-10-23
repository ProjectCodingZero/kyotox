use std::io::Error;
use std::ops::Deref;
use std::rc::Rc;
use crate::ky_async::Sender;
use thiserror::Error;
use vte::{Params, Perform};
use crate::grid::cell::Color::Ascii;
use crate::term::{EventTerminal, Terminal};
#[derive(Error, Debug)]
pub enum VteError {
    #[error("Send: {0}")]
    SendError(#[from] smol::channel::SendError<EventTerminal>),
    #[error("Parser error: {0}")]
    ParserError(String),
}
pub struct VteBridge{
    sender: Sender<EventTerminal>,
    errors: Vec<VteError>,
    parser: Rc<vte::Parser>,
}
impl std::fmt::Debug for VteBridge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("VteBridge")
            .field("sender", &"Sender<EventTerminal>")
            .field("errors", &self.errors)
            .field("parser", &"Parser")
            .finish()
    }
}
impl VteBridge {
    pub fn new(sender: Sender<EventTerminal>) -> Self {
        VteBridge{
            sender,
            errors: Vec::new(),
            parser: Rc::new(vte::Parser::new()),
        }
    }
    pub fn advance(&mut self, data: &[u8]) {
        let mut parser = Rc::clone(&self.parser);
        match Rc::get_mut(&mut parser) {
            Some(parser) => {
                parser.advance(self, data);
            },
            None => {
                self.add_error(VteError::ParserError("Failed to advance parser".to_string()));
            }
        }
    }
    fn call_once<F>(&mut self, func: F)
    where F: FnOnce() -> Result<(), VteError> {
        if let Err(err) = func() {
            self.errors.push(err)
        }
    }
    fn add_error(&mut self, error: VteError) {
        self.errors.push(error);
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
    use smol::channel::unbounded;
    use super::*;

    #[test]
    fn test_parser() {
        let (sender, receiver) = unbounded::<EventTerminal>();
        let mut vte = VteBridge::new(sender);
        let input_data = b"BAAAA";
        vte.advance(input_data);
        let event = receiver.recv_blocking().unwrap();
        assert_eq!(event, EventTerminal::PrintChar('A'));
    }
}