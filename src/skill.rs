use crate::config::{self, Config, InstallMode};
use crate::download;
use crate::marketplace;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Skill {
    pub name: String,
    pub path: PathBuf,
    pub remote_url: String,
    pub branch: String,
    #[serde(default)]
    pub mode: Option<InstallMode>,
}

pub fn install(
    name: &str,
    marketplace_name: Option<&str>,
    mode: Option<InstallMode>,
) -> Result<Skill, String> {
    let (_, entry) = marketplace::resolve_skill(name, marketplace_name)?;

    let store_dir = config::skills_store_dir();
    fs::create_dir_all(&store_dir)
        .map_err(|e| format!("Cannot create skills store: {e}"))?;

    let target = store_dir.join(name);

    if target.exists() {
        return Err(format!(
            "Skill '{name}' is already in the store. Use `ccsm update {name}` to refresh, or `ccsm remove {name}` first."
        ));
    }

    let branch = entry.branch.as_deref().unwrap_or("main");
    download::download_skill(&entry.repository, branch, &target)?;

    let deploy_source = match &entry.path {
        Some(subdir) => {
            let p = target.join(subdir);
            if !p.exists() {
                return Err(format!("Path '{}' not found in skill repository", subdir));
            }
            p
        }
        None => target.clone(),
    };

    let config = Config::load().map_err(|e| format!("Cannot read config: {e}"))?;
    let deploy_mode = mode.unwrap_or_else(|| config.default_mode.clone());
    deploy_skill(name, &deploy_source, &deploy_mode)?;

    Ok(Skill {
        name: name.to_string(),
        path: target,
        remote_url: entry.repository,
        branch: branch.to_string(),
        mode: Some(deploy_mode),
    })
}

fn deploy_skill(name: &str, source: &PathBuf, mode: &InstallMode) -> Result<(), String> {
    let claude_dir = config::claude_skills_dir();
    fs::create_dir_all(&claude_dir)
        .map_err(|e| format!("Cannot create Claude skills dir: {e}"))?;

    let dest = claude_dir.join(name);

    match mode {
        InstallMode::Copy => {
            if dest.exists() {
                fs::remove_dir_all(&dest).ok();
            }
            copy_dir_all(source, &dest)
                .map_err(|e| format!("Cannot copy skill to {}: {e}", dest.display()))?;
        }
        InstallMode::Link => {
            if dest.exists() {
                // Remove existing link or directory
                if dest.is_symlink() || dest.is_dir() {
                    fs::remove_dir_all(&dest).ok();
                    #[cfg(windows)]
                    {
                        // On Windows, directory symlinks may need junction handling
                        fs::remove_dir(&dest).ok();
                    }
                }
            }
            symlink_dir(source, &dest)?;
        }
    }

    Ok(())
}

fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn symlink_dir(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    std::os::unix::fs::symlink(src, dst)
        .map_err(|e| format!("Cannot create symlink: {e}"))
}

#[cfg(windows)]
fn symlink_dir(src: &PathBuf, dst: &PathBuf) -> Result<(), String> {
    std::os::windows::fs::symlink_dir(src, dst)
        .map_err(|e| format!("Cannot create symlink: {e}. Directory symlinks require Windows 10+ with Developer Mode enabled, or administrator privileges."))
}

pub fn list() -> Result<Vec<Skill>, String> {
    let dir = config::skills_store_dir();

    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = fs::read_dir(&dir).map_err(|e| format!("Cannot read skills store: {e}"))?;

    let mut skills = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("I/O error: {e}"))?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // We don't have git remotes anymore, so omit remote_url and branch for now
        let mode = detect_deploy_mode(&path);
        skills.push(Skill {
            name,
            path,
            remote_url: String::new(),
            branch: String::new(),
            mode,
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

pub fn remove(name: &str) -> Result<(), String> {
    let store_dir = config::skills_store_dir();
    let target = store_dir.join(name);

    // Remove from Claude skills if present (copy or link)
    let claude_target = config::claude_skills_dir().join(name);
    if claude_target.exists() {
        fs::remove_dir_all(&claude_target)
            .map_err(|e| format!("Cannot remove skill from Claude dir: {e}"))?;
    }

    // Remove from store
    if target.exists() {
        fs::remove_dir_all(&target)
            .map_err(|e| format!("Cannot remove skill from store: {e}"))?;
    } else {
        return Err(format!("Skill '{name}' is not installed."));
    }

    Ok(())
}

pub fn update(name: Option<&str>) -> Result<Vec<(String, String)>, String> {
    let store_dir = config::skills_store_dir();
    if !store_dir.exists() {
        return Err("No skills installed.".to_string());
    }

    let config = Config::load().map_err(|e| format!("Cannot read config: {e}"))?;

    let targets: Vec<(String, PathBuf, Option<InstallMode>)> = match name {
        Some(n) => {
            let p = store_dir.join(n);
            if !p.exists() {
                return Err(format!("Skill '{n}' is not installed."));
            }
            let mode = detect_deploy_mode(&p);
            vec![(n.to_string(), p, mode)]
        }
        None => {
            let entries =
                fs::read_dir(&store_dir).map_err(|e| format!("Cannot read skills store: {e}"))?;
            let mut paths = Vec::new();
            for entry in entries {
                let entry = entry.map_err(|e| format!("I/O error: {e}"))?;
                let path = entry.path();
                if path.is_dir() {
                    let skill_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let mode = detect_deploy_mode(&path);
                    paths.push((skill_name, path, mode));
                }
            }
            paths
        }
    };

    let mut results = Vec::new();

    for (skill_name, _path, deploy_mode) in &targets {
        // Find the skill in a marketplace to get its repo URL
        let manifest_skill = find_skill_in_marketplaces(skill_name, &config);

        match manifest_skill {
            Some((_mkt_entry, skill_entry)) => {
                let branch = skill_entry.branch.as_deref().unwrap_or("main");

                // Remove old version
                let old_path = store_dir.join(skill_name);
                if old_path.exists() {
                    fs::remove_dir_all(&old_path).ok();
                }

                match download::download_skill(&skill_entry.repository, branch, &old_path) {
                    Ok(()) => {
                        let deploy_source = match &skill_entry.path {
                            Some(subdir) => old_path.join(subdir),
                            None => old_path.clone(),
                        };
                        // Redeploy if previously deployed
                        if deploy_mode.is_some() || config.default_mode != InstallMode::Copy {
                            let mode = deploy_mode
                                .clone()
                                .unwrap_or_else(|| config.default_mode.clone());
                            match deploy_skill(skill_name, &deploy_source, &mode) {
                                Ok(()) => results.push((skill_name.clone(), "Updated".to_string())),
                                Err(e) => results.push((skill_name.clone(), format!("Downloaded but deploy failed: {e}"))),
                            }
                        } else {
                            results.push((skill_name.clone(), "Updated".to_string()));
                        }
                    }
                    Err(e) => results.push((skill_name.clone(), format!("Error: {e}"))),
                }
            }
            None => {
                results.push((
                    skill_name.clone(),
                    "Warning: skill not found in any marketplace; cannot update".to_string(),
                ));
            }
        }
    }

    Ok(results)
}

pub fn info(name: &str) -> Result<Skill, String> {
    let target = config::skills_store_dir().join(name);
    if !target.exists() {
        return Err(format!("Skill '{name}' is not installed."));
    }

    let mode = detect_deploy_mode(&target);
    Ok(Skill {
        name: name.to_string(),
        path: target,
        remote_url: String::new(),
        branch: String::new(),
        mode,
    })
}

fn detect_deploy_mode(path: &PathBuf) -> Option<InstallMode> {
    let claude_target = config::claude_skills_dir().join(
        path.file_name().unwrap_or_default(),
    );

    if !claude_target.exists() {
        return None;
    }

    if claude_target.is_symlink() {
        Some(InstallMode::Link)
    } else {
        Some(InstallMode::Copy)
    }
}

fn find_skill_in_marketplaces(
    name: &str,
    config: &Config,
) -> Option<(config::MarketplaceEntry, marketplace::SkillEntry)> {
    for entry in &config.marketplaces {
        let manifest = marketplace::load_cached_manifest(&entry.name).ok()?;
        for skill in &manifest.skills {
            if skill.name == name {
                return Some((entry.clone(), skill.clone()));
            }
        }
    }
    None
}
