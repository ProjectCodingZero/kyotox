use std::ops::Deref;
use std::sync::{Arc, Mutex};
use mlua::{prelude, FromLua, IntoLua, Lua, Table, UserData, UserDataMethods};
use color_eyre::eyre::{eyre, Result};
use smol::lock::RwLock;
use crate::{Settings};
use crate::tui::{AppConfig, SharedData};
pub struct ScriptEngineBridge{
    pub tx: smol::channel::Sender<SharedData>,
    pub state: Arc<RwLock<SharedData>>,
}
impl UserData for ScriptEngineBridge{
    fn add_methods<'lua, M: UserDataMethods<Self>>(methods: &mut M) {
        //Getter: config.get_refresh()
        methods.add_method("get_fps", |_, this, ()|{
            let fps= smol::block_on(async {
                this.state.read().await.fps
            });
            Ok(fps)
        });
        //Setter: config.set_refresh()
        methods.add_method_mut("set_fps", |_, this, fps: u64| {
            smol::block_on(async {
                this.state.write().await.fps = fps;
            });
            Ok(())
        });
        methods.add_method_mut("set_title", |_, this, title: String| {
            smol::block_on(async {
                this.state.write().await.title = title;
            });
            Ok(())
        });
        methods.add_method_mut("get_title", |_, this, ()| {
            let title = smol::block_on(async {
                this.state.read().await.title.clone()
            });
            Ok(title)
        })

    }
}

