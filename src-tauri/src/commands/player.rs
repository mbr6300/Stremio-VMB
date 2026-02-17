use serde::Serialize;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct ExternalPlayer {
    pub id: String,
    pub name: String,
    pub path: String,
    pub installed: bool,
}

const KNOWN_PLAYERS: &[(&str, &str, &[&str])] = &[
    ("vlc", "VLC", &[
        "/Applications/VLC.app",
        "/Applications/VLC.app/Contents/MacOS/VLC",
    ]),
    ("iina", "IINA", &[
        "/Applications/IINA.app",
    ]),
    ("mpv", "mpv", &[
        "/opt/homebrew/bin/mpv",
        "/usr/local/bin/mpv",
    ]),
];

fn find_player(id: &str) -> Option<ExternalPlayer> {
    KNOWN_PLAYERS.iter().find(|(pid, _, _)| *pid == id).and_then(|(pid, name, paths)| {
        for path in *paths {
            if Path::new(path).exists() {
                return Some(ExternalPlayer {
                    id: pid.to_string(),
                    name: name.to_string(),
                    path: path.to_string(),
                    installed: true,
                });
            }
        }
        None
    })
}

#[tauri::command]
pub fn detect_players() -> Vec<ExternalPlayer> {
    KNOWN_PLAYERS
        .iter()
        .filter_map(|(id, _, _)| find_player(id))
        .collect()
}

#[tauri::command]
pub fn open_in_player(player_id: String, file_path: String) -> Result<(), String> {
    let player = find_player(&player_id)
        .ok_or(format!("Player '{}' nicht gefunden", player_id))?;

    if player.path.ends_with(".app") {
        Command::new("open")
            .arg("-a")
            .arg(&player.path)
            .arg(&file_path)
            .spawn()
            .map_err(|e| format!("Fehler beim Starten von {}: {}", player.name, e))?;
    } else {
        Command::new(&player.path)
            .arg(&file_path)
            .spawn()
            .map_err(|e| format!("Fehler beim Starten von {}: {}", player.name, e))?;
    }

    Ok(())
}

#[tauri::command]
pub fn open_in_default_player(file_path: String) -> Result<(), String> {
    Command::new("open")
        .arg(&file_path)
        .spawn()
        .map_err(|e| format!("Fehler beim Öffnen: {}", e))?;
    Ok(())
}
