//! Musik-Metadaten: Scan, Tag-Extraktion (lofty), Cover-Speicherung.

use lofty::file::{AudioFile, TaggedFileExt};
use lofty::read_from_path;
use lofty::tag::Accessor;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const AUDIO_EXTENSIONS: &[&str] = &["mp3", "flac", "m4a", "aac", "ogg", "wma", "wav", "opus"];

fn is_audio_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|e| AUDIO_EXTENSIONS.contains(&e.to_lowercase().as_str()))
        .unwrap_or(false)
}

#[derive(Debug, Clone)]
pub struct MusicFile {
    pub file_path: String,
    pub file_hash: String,
    pub artist: String,
    pub album: String,
    pub title: String,
    pub track_number: Option<u32>,
    pub year: Option<u32>,
    pub duration_secs: u32,
    pub picture: Option<Vec<u8>>,
}

#[derive(Debug, serde::Serialize)]
pub struct MusicPathCheckResult {
    pub path: String,
    pub exists: bool,
    pub is_directory: bool,
    pub files_found: usize,
    pub sample_files: Vec<String>,
    pub sample_all: Vec<String>,
    pub subdirs: Vec<String>,
    pub error: Option<String>,
}

pub fn check_music_path(path: &str) -> MusicPathCheckResult {
    let path_str = path.trim();
    let p = Path::new(path_str);
    if !p.exists() {
        return MusicPathCheckResult {
            path: path_str.to_string(),
            exists: false,
            is_directory: false,
            files_found: 0,
            sample_files: vec![],
            sample_all: vec![],
            subdirs: vec![],
            error: Some("Pfad existiert nicht.".to_string()),
        };
    }
    if !p.is_dir() {
        return MusicPathCheckResult {
            path: path_str.to_string(),
            exists: true,
            is_directory: false,
            files_found: 0,
            sample_files: vec![],
            sample_all: vec![],
            subdirs: vec![],
            error: Some("Pfad ist kein Ordner.".to_string()),
        };
    }

    let mut files_found = 0;
    let mut sample_files = Vec::new();
    let mut sample_all = Vec::new();
    let mut subdirs = Vec::new();

    for entry in WalkDir::new(p).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && is_audio_file(path) {
            files_found += 1;
            if let Some(s) = path.to_str() {
                if sample_files.len() < 5 {
                    sample_files.push(s.to_string());
                }
                if sample_all.len() < 20 {
                    sample_all.push(s.to_string());
                }
            }
        } else if path.is_dir() && path != p && entry.depth() == 1 {
            if let Some(s) = path.file_name().and_then(|n| n.to_str()) {
                if !s.starts_with('.') {
                    subdirs.push(s.to_string());
                }
            }
        }
    }

    MusicPathCheckResult {
        path: path_str.to_string(),
        exists: true,
        is_directory: true,
        files_found,
        sample_files,
        sample_all,
        subdirs,
        error: None,
    }
}

fn file_hash(path: &Path) -> String {
    if let Ok(data) = std::fs::read(path) {
        format!("{:x}", Sha256::digest(&data))
    } else {
        format!("{:x}", Sha256::digest(path.to_string_lossy().as_bytes()))
    }
}

pub fn scan_music_directory(path: &str) -> Vec<MusicFile> {
    let mut results = Vec::new();
    let base = Path::new(path);

    for entry in WalkDir::new(base)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if !p.is_file() || !is_audio_file(p) {
            continue;
        }
        if let Some(mf) = extract_music_file(p) {
            results.push(mf);
        }
    }
    results
}

fn extract_music_file(path: &Path) -> Option<MusicFile> {
    let path_str = path.to_string_lossy().to_string();
    let file_hash = file_hash(path);
    let fallback_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unbekannt")
        .to_string();

    let tagged = match read_from_path(path) {
        Ok(t) => t,
        Err(_) => {
            return Some(MusicFile {
                file_path: path_str,
                file_hash,
                artist: "Unbekannt".to_string(),
                album: "Unbekannt".to_string(),
                title: fallback_name,
                track_number: None,
                year: None,
                duration_secs: 0,
                picture: None,
            });
        }
    };

    let duration_secs = tagged.properties().duration().as_secs() as u32;
    let mut artist = "Unbekannt".to_string();
    let mut album = "Unbekannt".to_string();
    let mut title = fallback_name.clone();
    let mut track_number = None;
    let mut year = None;
    let mut picture = None;

    if let Some(tag) = tagged.primary_tag().or_else(|| tagged.first_tag()) {
        if let Some(a) = tag.artist() {
            artist = a.to_string();
        }
        if let Some(al) = tag.album() {
            album = al.to_string();
        }
        if let Some(t) = tag.title() {
            title = t.to_string();
        }
        track_number = tag.track();
        if let Some(ts) = tag.date() {
            year = Some(ts.year as u32);
        }
        if let Some(pic) = tag.pictures().first() {
            picture = Some(pic.data().to_vec());
        }
    }

    if artist.is_empty() {
        artist = "Unbekannt".to_string();
    }
    if album.is_empty() {
        album = "Unbekannt".to_string();
    }
    if title.is_empty() {
        title = fallback_name;
    }

    Some(MusicFile {
        file_path: path_str,
        file_hash,
        artist,
        album,
        title,
        track_number,
        year,
        duration_secs,
        picture,
    })
}

/// Speichert Cover in cache_dir und gibt den Pfad zurück.
pub fn save_cover_to_cache(
    picture: &[u8],
    artist: &str,
    album: &str,
    cache_dir: &Path,
) -> Option<PathBuf> {
    std::fs::create_dir_all(cache_dir).ok()?;
    let safe_artist = artist.chars().filter(|c| c.is_alphanumeric() || *c == ' ').collect::<String>();
    let safe_album = album.chars().filter(|c| c.is_alphanumeric() || *c == ' ').collect::<String>();
    let hash = format!("{:x}", Sha256::digest(picture));
    let ext = if picture.len() > 8 {
        match &picture[0..4] {
            [0x89, 0x50, 0x4E, 0x47, ..] => "png",
            [0xFF, 0xD8, 0xFF, ..] => "jpg",
            _ => "jpg",
        }
    } else {
        "jpg"
    };
    let filename = format!("{}_{}_{}.{}", safe_artist, safe_album, &hash[..12], ext)
        .replace(' ', "_");
    let path = cache_dir.join(&filename);
    std::fs::write(&path, picture).ok()?;
    Some(path)
}
