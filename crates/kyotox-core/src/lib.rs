//! Temporary compatibility facade.
//!
//! The real implementations live in `kyotox-term`, `kyotox-session`,
//! `kyotox-pty` and `kyotox-config`. This crate only re-exports them so the
//! frontends can migrate incrementally.

pub mod cursor {
    pub use kyotox_term::Cursor;
}
pub mod grid {
    pub use kyotox_term::*;
}
pub mod logger {
    pub use kyotox_config::*;
}
pub mod parser {
    pub use kyotox_term::Screen;
}
pub mod pty {
    pub use kyotox_pty::*;
}
pub mod term {
    pub use kyotox_session::Terminal;
    pub use kyotox_term::*;
}
pub mod thread {
    pub use kyotox_config::*;
}

mod config {
    pub use kyotox_config::*;
}

pub use config::{CliConfig, Config, load_config, load_config_async};
