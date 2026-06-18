use crate::config::{self, Config, MarketplaceEntry};
use crate::download;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub skills: Vec<SkillEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SkillEntry {
    pub name: String,
    pub description: String,
    pub repository: String,
    #[serde(default)]
    pub branch: Option<String>,
}

pub fn add(url: &str, name: Option<&str>) -> Result<MarketplaceEntry, String> {
    let market_name = name
        .map(|n| n.to_string())
        .unwrap_or_else(|| repo_name_from_url(url));

    let mut config = Config::load().map_err(|e| format!("Cannot read config: {e}"))?;

    if config.marketplaces.iter().any(|m| m.name == market_name) {
        return Err(format!(
            "Marketplace '{market_name}' is already registered. Use `ccsm marketplace update {market_name}` to refresh."
        ));
    }

    let (json, branch) = download::fetch_marketplace_manifest(url)?;
    let manifest: Manifest =
        serde_json::from_str(&json).map_err(|e| format!("Invalid skills.json: {e}"))?;

    // Cache the manifest locally
    let cache_dir = config::marketplaces_cache_dir();
    fs::create_dir_all(&cache_dir)
        .map_err(|e| format!("Cannot create cache dir: {e}"))?;
    fs::write(cache_dir.join(format!("{market_name}.json")), &json)
        .map_err(|e| format!("Cannot cache manifest: {e}"))?;

    let entry = MarketplaceEntry {
        name: market_name.clone(),
        url: url.to_string(),
        branch: Some(branch),
    };
    config.marketplaces.push(entry.clone());
    config.save().map_err(|e| format!("Cannot save config: {e}"))?;

    println!(
        "Marketplace '{}' added ({} skills from \"{}\")",
        market_name,
        manifest.skills.len(),
        manifest.name
    );

    Ok(entry)
}

pub fn remove(name: &str) -> Result<(), String> {
    let mut config = Config::load().map_err(|e| format!("Cannot read config: {e}"))?;

    let pos = config
        .marketplaces
        .iter()
        .position(|m| m.name == name)
        .ok_or_else(|| format!("Marketplace '{name}' not found."))?;

    config.marketplaces.remove(pos);
    config.save().map_err(|e| format!("Cannot save config: {e}"))?;

    let cache_path = config::marketplaces_cache_dir().join(format!("{name}.json"));
    if cache_path.exists() {
        fs::remove_file(&cache_path).ok();
    }

    Ok(())
}

pub fn list() -> Result<Vec<(MarketplaceEntry, usize)>, String> {
    let config = Config::load().map_err(|e| format!("Cannot read config: {e}"))?;

    let mut result = Vec::new();
    for entry in &config.marketplaces {
        let manifest = load_cached_manifest(&entry.name);
        let count = manifest.map(|m| m.skills.len()).unwrap_or(0);
        result.push((entry.clone(), count));
    }

    Ok(result)
}

pub fn update(name: Option<&str>) -> Result<Vec<(String, String)>, String> {
    let config = Config::load().map_err(|e| format!("Cannot read config: {e}"))?;

    let targets: Vec<MarketplaceEntry> = match name {
        Some(n) => {
            let entry = config
                .marketplaces
                .iter()
                .find(|m| m.name == n)
                .ok_or_else(|| format!("Marketplace '{n}' not found."))?
                .clone();
            vec![entry]
        }
        None => config.marketplaces.clone(),
    };

    if targets.is_empty() {
        return Err("No marketplaces configured. Use `ccsm marketplace add <url>` first."
            .to_string());
    }

    let mut results = Vec::new();
    for entry in &targets {
        let name = entry.name.clone();
        match download::fetch_marketplace_manifest(&entry.url) {
            Ok((json, _branch)) => {
                let manifest: Manifest = match serde_json::from_str(&json) {
                    Ok(m) => m,
                    Err(e) => {
                        results.push((name, format!("Error: invalid skills.json — {e}")));
                        continue;
                    }
                };

                let cache_dir = config::marketplaces_cache_dir();
                fs::create_dir_all(&cache_dir).ok();
                fs::write(cache_dir.join(format!("{name}.json")), &json).ok();

                results.push((
                    name,
                    format!("Updated — {} skills available", manifest.skills.len()),
                ));
            }
            Err(e) => results.push((name, format!("Error: {e}"))),
        }
    }

    Ok(results)
}

pub fn search(query: Option<&str>) -> Result<Vec<(String, SkillEntry)>, String> {
    let config = Config::load().map_err(|e| format!("Cannot read config: {e}"))?;

    if config.marketplaces.is_empty() {
        return Err("No marketplaces configured. Use `ccsm marketplace add <url>` first."
            .to_string());
    }

    let mut results = Vec::new();
    let query_lower = query.map(|q| q.to_lowercase());

    for entry in &config.marketplaces {
        let manifest = match load_cached_manifest(&entry.name) {
            Ok(m) => m,
            Err(_) => continue,
        };

        for skill in manifest.skills {
            let matches = match &query_lower {
                Some(q) => {
                    skill.name.to_lowercase().contains(q)
                        || skill.description.to_lowercase().contains(q)
                }
                None => true,
            };

            if matches {
                results.push((entry.name.clone(), skill));
            }
        }
    }

    results.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    Ok(results)
}

pub fn resolve_skill(
    name: &str,
    marketplace: Option<&str>,
) -> Result<(MarketplaceEntry, SkillEntry), String> {
    let config = Config::load().map_err(|e| format!("Cannot read config: {e}"))?;

    let mut matches = Vec::new();

    for entry in &config.marketplaces {
        if let Some(ref mkt) = marketplace {
            if entry.name != *mkt {
                continue;
            }
        }

        let manifest = match load_cached_manifest(&entry.name) {
            Ok(m) => m,
            Err(_) => continue,
        };

        for skill in &manifest.skills {
            if skill.name == name {
                matches.push((entry.clone(), skill.clone()));
            }
        }
    }

    match matches.len() {
        0 => Err(format!(
            "Skill '{name}' not found in any marketplace. Use `ccsm search` to browse available skills."
        )),
        1 => Ok(matches.pop().unwrap()),
        _ => {
            let market_names: Vec<String> =
                matches.iter().map(|(m, _)| m.name.clone()).collect();
            Err(format!(
                "Skill '{name}' found in multiple marketplaces: {}. Use --marketplace to specify which one.",
                market_names.join(", ")
            ))
        }
    }
}

pub fn load_cached_manifest(name: &str) -> Result<Manifest, String> {
    let path = config::marketplaces_cache_dir().join(format!("{name}.json"));
    if !path.exists() {
        return Err(format!("Marketplace '{name}' cache not found. Run `ccsm marketplace update {name}`."));
    }
    let data = fs::read_to_string(&path).map_err(|e| format!("Cannot read cache: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("Invalid cached manifest: {e}"))
}

fn repo_name_from_url(url: &str) -> String {
    url.trim_end_matches('/')
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .unwrap_or("marketplace")
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_name_from_url_https() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/my-skill.git"),
            "my-skill"
        );
    }

    #[test]
    fn test_repo_name_from_url_no_git_suffix() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/my-skill"),
            "my-skill"
        );
    }

    #[test]
    fn test_repo_name_from_url_trailing_slash() {
        assert_eq!(
            repo_name_from_url("https://github.com/user/my-skill/"),
            "my-skill"
        );
    }
}
