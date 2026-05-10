use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use serde::Serialize;
use tauri::{Emitter, Manager};

struct OpenedUrls(Mutex<Vec<String>>);

#[derive(Serialize)]
struct FileRecord {
    path: String,
    name: String,
    contents: String,
}

#[derive(Serialize)]
struct SavedRecord {
    path: String,
    name: String,
}

fn filename(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("untitled.csv")
        .to_string()
}

fn path_from_url_or_path(input: &str) -> Result<PathBuf, String> {
    if let Ok(url) = tauri::Url::parse(input) {
        if url.scheme() == "file" {
            return url
                .to_file_path()
                .map_err(|_| "Could not convert file URL to a local path.".to_string());
        }
    }
    Ok(PathBuf::from(input))
}

fn read_path(path: PathBuf) -> Result<FileRecord, String> {
    let contents = fs::read_to_string(&path)
        .map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    Ok(FileRecord {
        name: filename(&path),
        path: path.to_string_lossy().to_string(),
        contents,
    })
}

fn save_path(path: PathBuf, contents: String) -> Result<SavedRecord, String> {
    fs::write(&path, contents)
        .map_err(|error| format!("Could not write {}: {error}", path.display()))?;
    Ok(SavedRecord {
        name: filename(&path),
        path: path.to_string_lossy().to_string(),
    })
}

fn file_like_arg(arg: &str) -> bool {
    let lower = arg.to_ascii_lowercase();
    !arg.starts_with("--")
        && (lower.ends_with(".csv")
            || lower.ends_with(".tsv")
            || lower.starts_with("file://"))
}

fn initial_file_args() -> Vec<String> {
    std::env::args().skip(1).filter(|arg| file_like_arg(arg)).collect()
}

fn store_and_emit(app: &tauri::AppHandle, urls: Vec<String>) {
    if urls.is_empty() {
        return;
    }
    app.state::<OpenedUrls>()
        .0
        .lock()
        .expect("opened URL state poisoned")
        .extend(urls.clone());
    let _ = app.emit("opened", urls);
}

#[tauri::command]
fn opened_urls(app: tauri::AppHandle) -> Vec<String> {
    let state = app.state::<OpenedUrls>();
    let mut urls = state
        .0
        .lock()
        .expect("opened URL state poisoned");
    let out = urls.clone();
    urls.clear();
    out
}

#[tauri::command]
fn read_opened_file(url: String) -> Result<FileRecord, String> {
    read_path(path_from_url_or_path(&url)?)
}

#[tauri::command]
fn open_file_dialog() -> Result<Option<FileRecord>, String> {
    match rfd::FileDialog::new()
        .add_filter("CSV and TSV", &["csv", "tsv", "txt"])
        .pick_file()
    {
        Some(path) => read_path(path).map(Some),
        None => Ok(None),
    }
}

#[tauri::command]
fn save_file(path: String, contents: String) -> Result<SavedRecord, String> {
    save_path(PathBuf::from(path), contents)
}

#[tauri::command]
fn save_file_dialog(suggested_name: String, contents: String) -> Result<Option<SavedRecord>, String> {
    match rfd::FileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name(suggested_name)
        .save_file()
    {
        Some(path) => save_path(path, contents).map(Some),
        None => Ok(None),
    }
}

fn main() {
    tauri::Builder::default()
        .manage(OpenedUrls(Mutex::new(Vec::new())))
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            let urls = argv
                .into_iter()
                .skip(1)
                .filter(|arg| file_like_arg(arg))
                .collect::<Vec<_>>();
            store_and_emit(app, urls);
        }))
        .invoke_handler(tauri::generate_handler![
            opened_urls,
            read_opened_file,
            open_file_dialog,
            save_file,
            save_file_dialog
        ])
        .setup(|app| {
            let args = initial_file_args();
            if !args.is_empty() {
                app.state::<OpenedUrls>()
                    .0
                    .lock()
                    .expect("opened URL state poisoned")
                    .extend(args);
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building Tauri application")
        .run(|app, event| {
            #[cfg(any(target_os = "macos", target_os = "ios", target_os = "android"))]
            if let tauri::RunEvent::Opened { urls } = event {
                let incoming = urls.into_iter().map(|url| url.to_string()).collect();
                store_and_emit(app, incoming);
            }
            #[cfg(not(any(target_os = "macos", target_os = "ios", target_os = "android")))]
            {
                let _ = app;
                let _ = event;
            }
        });
}
