use sha2::{Sha256, Digest};
use std::path::Path;
use walkdir::WalkDir;

fn normalize_path(path: &str) -> String {
    let path = path.trim();
    #[cfg(target_os = "macos")]
    {
        if path.starts_with("/volume/") {
            return format!("/Volumes/{}", path.trim_start_matches("/volume/"));
        }
        if path == "/volume" || path.starts_with("/volume") {
            return path.replacen("/volume", "/Volumes", 1);
        }
    }
    path.to_string()
}

fn expand_path(path: &str) -> String {
    let path = normalize_path(path);
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{}/{}", home.trim_end_matches('/'), path.trim_start_matches("~/"));
        }
    } else if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return home;
        }
    }
    path.to_string()
}

const VIDEO_EXTENSIONS: &[&str] = &[
    "mp4", "mkv", "avi", "mov", "wmv", "flv", "webm", "m4v", "mpg", "mpeg", "m2v", "m4p",
    "ts", "m2ts", "mts", "3gp", "3g2", "ogv", "vob", "iso", "divx", "xvid",
];

pub struct ScannedFile {
    pub file_path: String,
    pub title: String,
    pub file_size: i64,
    pub file_hash: String,
    pub media_type: String,
}

#[derive(serde::Serialize)]
pub struct PathCheckResult {
    pub path: String,
    pub exists: bool,
    pub is_directory: bool,
    pub files_found: usize,
    pub sample_files: Vec<String>,
    pub sample_all: Vec<String>,
    pub subdirs: Vec<String>,
    pub error: Option<String>,
}

pub fn check_path(path_str: &str) -> PathCheckResult {
    let path_str = path_str.trim();
    let expanded = expand_path(path_str);
    let path = Path::new(&expanded);
    if !path.exists() {
        return PathCheckResult {
            path: path_str.to_string(),
            exists: false,
            is_directory: false,
            files_found: 0,
            sample_files: vec![],
            sample_all: vec![],
            subdirs: vec![],
            error: Some(format!(
                "Pfad existiert nicht (geprüft: {}). NAS in Finder einbinden. macOS: App unter Systemeinstellungen → Datenschutz → Vollständiger Festplattenzugriff hinzufügen.",
                expanded
            )),
        };
    }
    if !path.is_dir() {
        return PathCheckResult {
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

    let mut files_found = 0usize;
    let mut sample_files = Vec::new();
    let mut sample_all = Vec::new();
    let mut subdirs = Vec::new();

    for entry in WalkDir::new(path).follow_links(true).into_iter().filter_map(|e| e.ok()) {
        let entry_path = entry.path();
        if entry_path.is_dir() {
            if entry.depth() == 1 {
                if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                    subdirs.push(name.to_string());
                }
            }
            continue;
        }
        if !entry_path.is_file() {
            continue;
        }
        let ext = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if sample_all.len() < 10 {
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                sample_all.push(name.to_string());
            }
        }
        if !VIDEO_EXTENSIONS.contains(&ext.as_str()) {
            continue;
        }
        files_found += 1;
        if sample_files.len() < 5 {
            if let Some(name) = entry_path.file_name().and_then(|n| n.to_str()) {
                sample_files.push(name.to_string());
            }
        }
    }

    PathCheckResult {
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

pub fn scan_directories(paths: &[String]) -> Vec<ScannedFile> {
    let mut results = Vec::new();

    for dir_path in paths {
        let expanded = expand_path(dir_path);
        let path = Path::new(&expanded);
        if !path.exists() {
            log::warn!("Medienpfad existiert nicht: {}", dir_path);
            continue;
        }
        if !path.is_dir() {
            log::warn!("Medienpfad ist kein Ordner: {}", dir_path);
            continue;
        }
        log::info!("Scanne Medienpfad (inkl. Unterordner): {}", dir_path);

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| match e {
                Ok(entry) => Some(entry),
                Err(err) => {
                    let path_str = err.path().map(|p| p.display().to_string()).unwrap_or_default();
                    log::warn!("Scan error: {} - {}", path_str, err);
                    None
                }
            })
        {
            let entry_path = entry.path();
            if !entry_path.is_file() {
                continue;
            }

            let ext = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !VIDEO_EXTENSIONS.contains(&ext.as_str()) {
                continue;
            }

            let file_size = std::fs::metadata(entry_path)
                .map(|m| m.len() as i64)
                .unwrap_or(0);

            let file_hash = compute_partial_hash(entry_path).unwrap_or_default();
            let title = extract_title(entry_path);
            let media_type = detect_media_type(entry_path);

            results.push(ScannedFile {
                file_path: entry_path.to_string_lossy().to_string(),
                title,
                file_size,
                file_hash,
                media_type,
            });
        }
    }

    results
}

pub fn scan_directories_streaming<F>(paths: &[String], mut on_file: F)
where
    F: FnMut(ScannedFile),
{
    for dir_path in paths {
        let expanded = expand_path(dir_path);
        let path = Path::new(&expanded);
        if !path.exists() {
            log::warn!("Medienpfad existiert nicht: {}", dir_path);
            continue;
        }
        if !path.is_dir() {
            log::warn!("Medienpfad ist kein Ordner: {}", dir_path);
            continue;
        }
        log::info!("Scanne Medienpfad (inkl. Unterordner): {}", dir_path);

        for entry in WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| match e {
                Ok(entry) => Some(entry),
                Err(err) => {
                    let path_str = err.path().map(|p| p.display().to_string()).unwrap_or_default();
                    log::warn!("Scan error: {} - {}", path_str, err);
                    None
                }
            })
        {
            let entry_path = entry.path();
            if !entry_path.is_file() {
                continue;
            }

            let ext = entry_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            if !VIDEO_EXTENSIONS.contains(&ext.as_str()) {
                continue;
            }

            let file_size = std::fs::metadata(entry_path)
                .map(|m| m.len() as i64)
                .unwrap_or(0);

            let file_hash = compute_partial_hash(entry_path).unwrap_or_default();
            let title = extract_title(entry_path);
            let media_type = detect_media_type(entry_path);

            on_file(ScannedFile {
                file_path: entry_path.to_string_lossy().to_string(),
                title,
                file_size,
                file_hash,
                media_type,
            });
        }
    }
}

fn compute_partial_hash(path: &Path) -> Result<String, std::io::Error> {
    use std::io::Read;
    let mut file = std::fs::File::open(path)?;
    let mut buffer = vec![0u8; 1024 * 1024]; // 1 MB
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);

    let mut hasher = Sha256::new();
    hasher.update(&buffer);
    Ok(format!("{:x}", hasher.finalize()))
}

fn extract_title(path: &Path) -> String {
    path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .replace('.', " ")
        .replace('_', " ")
        .replace('-', " ")
}

fn detect_media_type(path: &Path) -> String {
    let path_str = path.to_string_lossy().to_lowercase();
    // S01E01, S1E1, 1x01, 2x05, Episode 1, Season 1, Staffel 1, Folge 1
    let indicators = [
        "season", "staffel", "episode", "ep ", "folge",
        "s01", "s02", "s03", "s04", "s05", "s06", "s07", "s08", "s09", "s10", "s1e", "s2e", "s3e",
        "e01", "e02", "e03", "e04", "e05", "e06", "e07", "e08", "e09", "e10",
        "1x01", "1x02", "2x01", "3x01", "x01", "x02", "x03", "x04", "x05",
    ];
    if indicators.iter().any(|ind| path_str.contains(ind)) {
        "series".to_string()
    } else {
        "movie".to_string()
    }
}
