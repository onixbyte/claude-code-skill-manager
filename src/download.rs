use std::fs;
use std::io::{self, Cursor, Read};
use std::path::Path;

pub fn get_json(url: &str) -> Result<String, String> {
    let resp = reqwest::blocking::get(url)
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {status} from {url}"));
    }

    resp.text()
        .map_err(|e| format!("Cannot read response body: {e}"))
}

pub fn fetch_marketplace_manifest(repo_url: &str) -> Result<(String, String), String> {
    let base = repo_url.trim_end_matches('/').trim_end_matches(".git");

    for branch in &["main", "master"] {
        let url = format!("{base}/raw/branch/{branch}/skills.json");
        match get_json(&url) {
            Ok(json) => return Ok((json, branch.to_string())),
            Err(_) => continue,
        }
    }

    Err(format!(
        "Cannot fetch skills.json from {base}. Tried branches 'main' and 'master'.\n\
         Make sure the repository is public and contains skills.json at its root."
    ))
}

pub fn download_skill(repo_url: &str, branch: &str, dest: &Path) -> Result<(), String> {
    let base = repo_url.trim_end_matches('/').trim_end_matches(".git");

    let mut last_err = String::new();

    for ext in &["zip", "tar.gz"] {
        let url = format!("{base}/archive/{branch}.{ext}");
        match download_and_extract(&url, dest) {
            Ok(()) => return Ok(()),
            Err(e) => {
                last_err = e;
                continue;
            }
        }
    }

    Err(format!("Cannot download skill from {base}: {last_err}"))
}

fn download_and_extract(url: &str, dest: &Path) -> Result<(), String> {
    let resp = reqwest::blocking::get(url)
        .map_err(|e| format!("HTTP request failed: {e}"))?;

    let status = resp.status();
    if !status.is_success() {
        return Err(format!("HTTP {status}"));
    }

    let bytes = resp
        .bytes()
        .map_err(|e| format!("Cannot read response body: {e}"))?;

    if url.ends_with(".zip") {
        extract_zip(&bytes, dest)
    } else if url.ends_with(".tar.gz") {
        extract_tar_gz(&bytes, dest)
    } else if bytes.len() >= 2 && &bytes[..2] == b"PK" {
        extract_zip(&bytes, dest)
    } else if bytes.len() >= 2 && &bytes[..2] == b"\x1f\x8b" {
        extract_tar_gz(&bytes, dest)
    } else {
        Err("Unknown archive format".to_string())
    }
}

fn extract_zip(data: &[u8], dest: &Path) -> Result<(), String> {
    let cursor = Cursor::new(data);
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Cannot open zip: {e}"))?;

    let prefix = detect_strip_prefix_zip(&mut archive);

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("Cannot read zip entry: {e}"))?;

        let name = file.name().to_string();
        let entry_path = match &prefix {
            Some(p) => match name.strip_prefix(p) {
                Some(rest) if !rest.is_empty() => rest,
                _ => continue,
            },
            None => &name,
        };

        let out_path = dest.join(entry_path);

        if file.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("Cannot create dir {}: {e}", out_path.display()))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create dir {}: {e}", parent.display()))?;
            }
            let mut out = fs::File::create(&out_path)
                .map_err(|e| format!("Cannot create file {}: {e}", out_path.display()))?;
            io::copy(&mut file, &mut out)
                .map_err(|e| format!("Cannot write file {}: {e}", out_path.display()))?;
        }
    }

    Ok(())
}

fn extract_tar_gz(data: &[u8], dest: &Path) -> Result<(), String> {
    let decoder = flate2::read::GzDecoder::new(data);
    let mut archive = tar::Archive::new(decoder);

    let entries: Vec<_> = archive
        .entries()
        .map_err(|e| format!("Cannot read tar: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("Cannot read tar entry: {e}"))?;

    let prefix = detect_strip_prefix_tar(&entries);

    for mut entry in entries {
        let path = entry
            .path()
            .map_err(|e| format!("Cannot read entry path: {e}"))?;
        let path_str = path.to_string_lossy().to_string();

        let entry_path = match &prefix {
            Some(p) => match path_str.strip_prefix(p) {
                Some(rest) if !rest.is_empty() => rest,
                _ => continue,
            },
            None => &path_str,
        };

        let out_path = dest.join(entry_path);

        if entry.header().entry_type().is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("Cannot create dir {}: {e}", out_path.display()))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("Cannot create dir {}: {e}", parent.display()))?;
            }
            entry
                .unpack(&out_path)
                .map_err(|e| format!("Cannot unpack {}: {e}", out_path.display()))?;
        }
    }

    Ok(())
}

fn detect_strip_prefix_zip(archive: &mut zip::ZipArchive<Cursor<&[u8]>>) -> Option<String> {
    let mut top_dirs: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(file) = archive.by_index(i) {
            let name = file.name();
            if let Some(slash_pos) = name.find('/') {
                let top = &name[..=slash_pos];
                if top_dirs.is_empty() || top_dirs[0] == top {
                    top_dirs.push(top.to_string());
                } else {
                    return None;
                }
            } else if !name.is_empty() {
                return None;
            }
        }
    }
    if top_dirs.len() >= 2 {
        Some(top_dirs[0].clone())
    } else {
        None
    }
}

fn detect_strip_prefix_tar(entries: &[tar::Entry<impl Read>]) -> Option<String> {
    let mut prefix: Option<String> = None;
    for entry in entries {
        if let Ok(path) = entry.path() {
            let path_str = path.to_string_lossy().to_string();
            if let Some(slash_pos) = path_str.find('/') {
                let top = &path_str[..=slash_pos];
                match &prefix {
                    Some(p) if p != top => return None,
                    None => prefix = Some(top.to_string()),
                    _ => {}
                }
            } else if !path_str.is_empty() {
                return None;
            }
        }
    }
    prefix
}
