use std::sync::Arc;
use parking_lot::Mutex;
pub mod command;
pub mod tui;
pub mod lua;

use tui::SharedData;
use crate::tui::AppConfig;

#[derive(Debug)]
struct Settings{
    pub app: AppConfig,
}



