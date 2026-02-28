#[derive(Debug)]
pub struct App {
    pub cursor_index: usize,
    pub title: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            cursor_index: 0,
            title: String::from("Herrumbre"),
        }
    }
}

impl App {
    pub fn run(&mut self) {}
}
