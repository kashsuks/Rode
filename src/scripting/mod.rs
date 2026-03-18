use mlua::{Lua, Result as LuaResult};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub enum EditorCommand {
    UseBuiltinTheme(String),
    SetThemeColor { name: String, value: String },
    SetSidebarVisible(bool),
    SetSidebarWidth(f32),
}

#[derive(Debug, Clone)]
pub struct StartupScriptLoad {
    pub path: PathBuf,
    pub source: Option<String>,
    pub commands: Vec<EditorCommand>,
    pub error: Option<String>,
}

pub fn startup_script_path() -> PathBuf {
    crate::config::theme_manager::get_config_dir().join("init.lua")
}

pub fn load_startup_script() -> StartupScriptLoad {
    let path = startup_script_path();

    let source = match fs::read_to_string(&path) {
        Ok(source) => source,
        Err(err) => {
            return StartupScriptLoad {
                path,
                source: None,
                commands: Vec::new(),
                error: Some(err.to_string()),
            };
        }
    };

    match eval_script(&source) {
        Ok(commands) => StartupScriptLoad {
            path,
            source: Some(source),
            commands,
            error: None,
        },
        Err(err) => StartupScriptLoad {
            path,
            source: Some(source),
            commands: Vec::new(),
            error: Some(err),
        },
    }
}

pub fn eval_script(source: &str) -> Result<Vec<EditorCommand>, String> {
    let lua = Lua::new();
    let commands = Arc::new(Mutex::new(Vec::<EditorCommand>::new()));

    let pinel = lua.create_table().map_err(|e| e.to_string())?;
    let theme = lua.create_table().map_err(|e| e.to_string())?;
    let ui = lua.create_table().map_err(|e| e.to_string())?;

    {
        let commands = Arc::clone(&commands);
        let f = lua
            .create_function(move |_, name: String| -> LuaResult<()> {
                commands
                    .lock()
                    .unwrap()
                    .push(EditorCommand::UseBuiltinTheme(name));
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        theme.set("use_builtin", f).map_err(|e| e.to_string())?;
    }

    {
        let commands = Arc::clone(&commands);
        let f = lua
            .create_function(move |_, (name, value): (String, String)| -> LuaResult<()> {
                commands
                    .lock()
                    .unwrap()
                    .push(EditorCommand::SetThemeColor { name, value });
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        theme.set("set_color", f).map_err(|e| e.to_string())?;
    }

    {
        let commands = Arc::clone(&commands);
        let f = lua
            .create_function(move |_, visible: bool| -> LuaResult<()> {
                commands
                    .lock()
                    .unwrap()
                    .push(EditorCommand::SetSidebarVisible(visible));
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        ui.set("show_sidebar", f).map_err(|e| e.to_string())?;
    }

    {
        let commands = Arc::clone(&commands);
        let f = lua
            .create_function(move |_, width: f32| -> LuaResult<()> {
                commands
                    .lock()
                    .unwrap()
                    .push(EditorCommand::SetSidebarWidth(width));
                Ok(())
            })
            .map_err(|e| e.to_string())?;
        ui.set("set_sidebar_width", f).map_err(|e| e.to_string())?;
    }

    pinel.set("theme", theme).map_err(|e| e.to_string())?;
    pinel.set("ui", ui).map_err(|e| e.to_string())?;
    lua.globals().set("pinel", pinel).map_err(|e| e.to_string())?;

    lua.load(source).exec().map_err(|e| e.to_string())?;

    let queued_commands = commands.lock().unwrap().clone();
    Ok(queued_commands)
}
