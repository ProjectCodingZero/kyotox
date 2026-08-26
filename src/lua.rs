//! Lua scripting engine bridge and inter-process state binding via `mlua`.

use crate::tui::SharedData;
use log::trace;
use mlua::{UserData, UserDataMethods};
use smol::lock::RwLock;
use std::sync::Arc;

/// Bridge struct providing access to shared application state within Lua scripts.
///
/// Binds methods to the global `app` object in Lua scripts to query and mutate
/// application properties such as title and FPS frame timing.
pub struct ScriptEngineBridge {
    /// Channel sender for dispatching updated [`SharedData`] state.
    pub tx: smol::channel::Sender<SharedData>,
    /// Thread-safe shared state handle locked with `smol::lock::RwLock`.
    pub state: Arc<RwLock<SharedData>>,
}

impl UserData for ScriptEngineBridge {
    /// Registers Lua methods on the `app` object.
    ///
    /// Available methods in Lua scripts:
    /// - `app:get_fps()` -> Returns current target FPS polling duration in ms.
    /// - `app:set_fps(fps)` -> Sets target FPS polling duration in ms.
    /// - `app:set_title(title)` -> Sets application window/block title.
    /// - `app:get_title()` -> Returns current application title.
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        // Getter: app:get_fps()
        methods.add_method("get_fps", |_, this, ()| {
            let fps = smol::block_on(async { this.state.read().await.fps });
            Ok(fps)
        });
        // Setter: app:set_fps(fps)
        methods.add_method_mut("set_fps", |_, this, fps: u64| {
            smol::block_on(async {
                this.state.write().await.fps = fps;
            });
            Ok(())
        });
        // Setter: app:set_title(title)
        methods.add_method_mut("set_title", |_, this, title: String| {
            smol::block_on(async {
                this.state.write().await.title = title;
            });
            Ok(())
        });
        // Getter: app:get_title()
        methods.add_method_mut("get_title", |_, this, ()| {
            let title = smol::block_on(async { this.state.read().await.title.clone() });
            Ok(title)
        });
    }
}
