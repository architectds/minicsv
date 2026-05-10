use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use encoding_rs::{Encoding, BIG5, EUC_KR, GB18030, SHIFT_JIS, WINDOWS_1252};
use serde::Serialize;
use tauri::{Emitter, Manager};

struct OpenedUrls(Mutex<Vec<String>>);

#[derive(Serialize)]
struct FileRecord {
    path: String,
    name: String,
    contents: String,
    encoding: String,
}

#[derive(Serialize)]
struct SavedRecord {
    path: String,
    name: String,
    encoding: String,
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

fn canonical_encoding(label: Option<&str>) -> String {
    let raw = label
        .unwrap_or("utf-8")
        .trim()
        .to_ascii_lowercase()
        .replace('_', "-");

    match raw.as_str() {
        "" | "auto" | "utf8" | "utf-8" => "utf-8",
        "utf8-bom" | "utf-8-bom" | "utf-8-sig" => "utf-8-bom",
        "utf16le" | "utf-16le" | "utf-16-le" => "utf-16le",
        "utf16be" | "utf-16be" | "utf-16-be" => "utf-16be",
        "latin1" | "latin-1" | "iso-8859-1" | "windows1252" | "windows-1252" => "windows-1252",
        "gbk" | "gb18030" => "gb18030",
        "big5" => "big5",
        "shiftjis" | "shift-jis" | "shift_jis" | "sjis" => "shift_jis",
        "euckr" | "euc-kr" => "euc-kr",
        _ => "utf-8",
    }
    .to_string()
}

fn display_encoding(encoding: &str) -> &'static str {
    match encoding {
        "utf-8" => "UTF-8",
        "utf-8-bom" => "UTF-8 with BOM",
        "utf-16le" => "UTF-16 LE",
        "utf-16be" => "UTF-16 BE",
        "windows-1252" => "Windows-1252",
        "gb18030" => "GB18030",
        "big5" => "Big5",
        "shift_jis" => "Shift_JIS",
        "euc-kr" => "EUC-KR",
        _ => "UTF-8",
    }
}

fn legacy_encoding(encoding: &str) -> Option<&'static Encoding> {
    match encoding {
        "windows-1252" => Some(WINDOWS_1252),
        "gb18030" => Some(GB18030),
        "big5" => Some(BIG5),
        "shift_jis" => Some(SHIFT_JIS),
        "euc-kr" => Some(EUC_KR),
        _ => None,
    }
}

fn selected_encoding(bytes: &[u8], requested: Option<&str>) -> (String, usize) {
    let requested = requested.unwrap_or("").trim();
    let requested_encoding = canonical_encoding(Some(requested));

    if requested.is_empty()
        || requested.eq_ignore_ascii_case("auto")
        || matches!(requested_encoding.as_str(), "utf-8" | "utf-8-bom")
    {
        if bytes.starts_with(&[0xef, 0xbb, 0xbf]) {
            return ("utf-8-bom".to_string(), 3);
        }
        if bytes.starts_with(&[0xff, 0xfe]) {
            return ("utf-16le".to_string(), 2);
        }
        if bytes.starts_with(&[0xfe, 0xff]) {
            return ("utf-16be".to_string(), 2);
        }
        return ("utf-8".to_string(), 0);
    }

    let encoding = requested_encoding;
    let skip = match encoding.as_str() {
        "utf-8" | "utf-8-bom" if bytes.starts_with(&[0xef, 0xbb, 0xbf]) => 3,
        "utf-16le" if bytes.starts_with(&[0xff, 0xfe]) => 2,
        "utf-16be" if bytes.starts_with(&[0xfe, 0xff]) => 2,
        _ => 0,
    };
    (encoding, skip)
}

fn decode_utf16(data: &[u8], little_endian: bool) -> Result<String, String> {
    if data.len() % 2 != 0 {
        return Err("UTF-16 data has an odd byte length.".to_string());
    }

    let units = data
        .chunks_exact(2)
        .map(|chunk| {
            if little_endian {
                u16::from_le_bytes([chunk[0], chunk[1]])
            } else {
                u16::from_be_bytes([chunk[0], chunk[1]])
            }
        })
        .collect::<Vec<_>>();

    String::from_utf16(&units).map_err(|error| format!("Could not decode UTF-16 text: {error}"))
}

fn decode_bytes(bytes: &[u8], requested: Option<&str>) -> Result<(String, String), String> {
    let (encoding, skip) = selected_encoding(bytes, requested);
    let data = &bytes[skip..];

    let contents = match encoding.as_str() {
        "utf-8" | "utf-8-bom" => String::from_utf8(data.to_vec())
            .map_err(|error| format!("Could not decode as UTF-8: {error}"))?,
        "utf-16le" => decode_utf16(data, true)?,
        "utf-16be" => decode_utf16(data, false)?,
        _ => {
            let Some(encoding_rs) = legacy_encoding(&encoding) else {
                return Err(format!("Unsupported encoding: {encoding}"));
            };
            let (text, _, had_errors) = encoding_rs.decode(data);
            if had_errors {
                return Err(format!(
                    "Could not decode file as {}.",
                    display_encoding(&encoding)
                ));
            }
            text.into_owned()
        }
    };

    Ok((contents, encoding))
}

fn encode_utf16(contents: &str, little_endian: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(contents.len() * 2);
    for unit in contents.encode_utf16() {
        let bytes = if little_endian {
            unit.to_le_bytes()
        } else {
            unit.to_be_bytes()
        };
        out.extend_from_slice(&bytes);
    }
    out
}

fn encode_text(contents: &str, encoding: Option<&str>) -> Result<(Vec<u8>, String), String> {
    let encoding = canonical_encoding(encoding);
    let bytes = match encoding.as_str() {
        "utf-8" => contents.as_bytes().to_vec(),
        "utf-8-bom" => {
            let mut out = vec![0xef, 0xbb, 0xbf];
            out.extend_from_slice(contents.as_bytes());
            out
        }
        "utf-16le" => encode_utf16(contents, true),
        "utf-16be" => encode_utf16(contents, false),
        _ => {
            let Some(encoding_rs) = legacy_encoding(&encoding) else {
                return Err(format!("Unsupported encoding: {encoding}"));
            };
            let (encoded, _, had_errors) = encoding_rs.encode(contents);
            if had_errors {
                return Err(format!(
                    "This file contains characters that cannot be saved as {}.",
                    display_encoding(&encoding)
                ));
            }
            encoded.into_owned()
        }
    };

    Ok((bytes, encoding))
}

fn read_path(path: PathBuf, encoding: Option<String>) -> Result<FileRecord, String> {
    let bytes =
        fs::read(&path).map_err(|error| format!("Could not read {}: {error}", path.display()))?;
    let (contents, encoding) = decode_bytes(&bytes, encoding.as_deref())?;
    Ok(FileRecord {
        name: filename(&path),
        path: path.to_string_lossy().to_string(),
        contents,
        encoding,
    })
}

fn save_path(
    path: PathBuf,
    contents: String,
    encoding: Option<String>,
) -> Result<SavedRecord, String> {
    let (bytes, encoding) = encode_text(&contents, encoding.as_deref())?;
    fs::write(&path, bytes)
        .map_err(|error| format!("Could not write {}: {error}", path.display()))?;
    Ok(SavedRecord {
        name: filename(&path),
        path: path.to_string_lossy().to_string(),
        encoding,
    })
}

fn file_like_arg(arg: &str) -> bool {
    let lower = arg.to_ascii_lowercase();
    !arg.starts_with("--")
        && (lower.ends_with(".csv") || lower.ends_with(".tsv") || lower.starts_with("file://"))
}

fn initial_file_args() -> Vec<String> {
    std::env::args()
        .skip(1)
        .filter(|arg| file_like_arg(arg))
        .collect()
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
    let mut urls = state.0.lock().expect("opened URL state poisoned");
    let out = urls.clone();
    urls.clear();
    out
}

#[tauri::command]
fn read_opened_file(url: String, encoding: Option<String>) -> Result<FileRecord, String> {
    read_path(path_from_url_or_path(&url)?, encoding)
}

#[tauri::command]
fn open_file_dialog(encoding: Option<String>) -> Result<Option<FileRecord>, String> {
    match rfd::FileDialog::new()
        .add_filter("CSV and TSV", &["csv", "tsv", "txt"])
        .pick_file()
    {
        Some(path) => read_path(path, encoding).map(Some),
        None => Ok(None),
    }
}

#[tauri::command]
fn save_file(
    path: String,
    contents: String,
    encoding: Option<String>,
) -> Result<SavedRecord, String> {
    save_path(PathBuf::from(path), contents, encoding)
}

#[tauri::command]
fn save_file_dialog(
    suggested_name: String,
    contents: String,
    encoding: Option<String>,
) -> Result<Option<SavedRecord>, String> {
    match rfd::FileDialog::new()
        .add_filter("CSV", &["csv"])
        .set_file_name(suggested_name)
        .save_file()
    {
        Some(path) => save_path(path, contents, encoding).map(Some),
        None => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_utf8_bom() {
        let bytes = [0xef, 0xbb, 0xbf, b'a', b',', b'b'];
        let (decoded, encoding) = decode_bytes(&bytes, None).expect("decode utf-8 bom");
        assert_eq!(decoded, "a,b");
        assert_eq!(encoding, "utf-8-bom");
    }

    #[test]
    fn round_trips_utf16le() {
        let text = "alpha,beta\n1,2";
        let (bytes, encoding) = encode_text(text, Some("utf-16le")).expect("encode utf-16le");
        let (decoded, decoded_encoding) =
            decode_bytes(&bytes, Some("utf-16le")).expect("decode utf-16le");
        assert_eq!(encoding, "utf-16le");
        assert_eq!(decoded_encoding, "utf-16le");
        assert_eq!(decoded, text);
    }

    #[test]
    fn detects_utf16le_bom_in_default_mode() {
        let mut bytes = vec![0xff, 0xfe];
        bytes.extend_from_slice(&encode_utf16("a,b", true));
        let (decoded, encoding) = decode_bytes(&bytes, Some("utf-8")).expect("decode utf-16le bom");
        assert_eq!(decoded, "a,b");
        assert_eq!(encoding, "utf-16le");
    }

    #[test]
    fn round_trips_gb18030() {
        let text = "\u{4e2d}\u{6587},CSV\n1,2";
        let (bytes, encoding) = encode_text(text, Some("gb18030")).expect("encode gb18030");
        let (decoded, decoded_encoding) =
            decode_bytes(&bytes, Some("gb18030")).expect("decode gb18030");
        assert_eq!(encoding, "gb18030");
        assert_eq!(decoded_encoding, "gb18030");
        assert_eq!(decoded, text);
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
