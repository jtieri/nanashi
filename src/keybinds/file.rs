use std::{env, fs, io, path::Path};

use super::Keybinds;

/// Read the keybinds file from the config directory, creating it with defaults
/// if it does not exist. On first run, an existing tui-chan config is carried
/// over so anyone coming from the original keeps their keybinds.
pub fn read_or_create_keybinds_file() -> Result<String, io::Error> {
    let Ok(config) = get_config_folder() else {
        eprintln!("Could not find the home config folder. Continuing with the default config.");
        return Ok(Keybinds::default_file_contents());
    };

    let folder = format!("{config}/nanashi");
    let filepath = format!("{folder}/keybinds.conf");

    if !Path::new(&folder).exists() {
        fs::create_dir(&folder)?;
    }

    // A nanashi config already exists, use it.
    if Path::new(&filepath).exists() {
        return fs::read_to_string(&filepath);
    }

    // First run: carry over an old tui-chan config if there is one, otherwise
    // start from the defaults. Either way write it to the nanashi path.
    let legacy = format!("{config}/tui-chan/keybinds.conf");
    let contents =
        fs::read_to_string(&legacy).unwrap_or_else(|_| Keybinds::default_file_contents());
    fs::write(&filepath, &contents)?;
    Ok(contents)
}

/// Get config home folder for Linux
fn get_config_folder() -> Result<String, env::VarError> {
    env::var("XDG_CONFIG_HOME")
        .or_else(|_| env::var("HOME").map(|home| format!("{}/.config", home)))
}
