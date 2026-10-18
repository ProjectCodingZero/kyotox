use ratatui::{DefaultTerminal, prelude::*};
use smol::io;

#[derive(Debug, Default)]
pub struct App{
    exit: bool
}

impl App{
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()>{
        while !self.exit{
            terminal.draw(|frame| self.draw(frame));
            self.handle_events()?
        }
        Ok(())
    }
    fn draw(&self, frame: &mut Frame){
        todo!()
    }
    fn handle_events(&mut self) -> io::Result<()>{
        todo!()
    }
}