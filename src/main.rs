mod cli;
mod config;
mod download;
mod marketplace;
mod skill;

use clap::Parser;
use cli::{Cli, Command, MarketplaceCommand};
use config::InstallMode;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::Marketplace(cmd) => run_marketplace(cmd),
        Command::Search(args) => {
            let results = marketplace::search(args.query.as_deref())?;
            if results.is_empty() {
                println!("No skills found.");
                if args.query.is_none() {
                    println!(
                        "No marketplaces configured, or all marketplaces are empty."
                    );
                    println!("Use `ccsm marketplace add <url>` to add a marketplace.");
                }
            } else {
                println!("Found {} skill(s):\n", results.len());
                for (market, skill) in &results {
                    println!("  {}  ({})", skill.name, market);
                    println!("      {}\n", skill.description);
                }
            }
            Ok(())
        }
        Command::Install(args) => {
            let mode = if args.link {
                Some(InstallMode::Link)
            } else if args.copy {
                Some(InstallMode::Copy)
            } else {
                None
            };

            let mode_label = match &mode {
                Some(InstallMode::Link) => " (link)",
                Some(InstallMode::Copy) => " (copy)",
                None => "",
            };

            let s = skill::install(&args.name, args.marketplace.as_deref(), mode)?;
            println!(
                "Installed skill '{}'{}",
                s.name, mode_label
            );
            println!("  Store:   {}", s.path.display());
            println!("  Claude:  {}", config::claude_skills_dir().join(&s.name).display());
            println!("  Remote:  {}", s.remote_url);
            println!("  Branch:  {}", s.branch);
            Ok(())
        }
        Command::List => {
            let skills = skill::list()?;
            if skills.is_empty() {
                println!("No skills installed.");
                println!("Store: {}", config::skills_store_dir().display());
                println!(
                    "Use `ccsm search` to browse available skills, then `ccsm install <name>`."
                );
            } else {
                println!("Installed skills ({})\n", skills.len());
                for s in &skills {
                    let has_manifest = has_skill_manifest(&s.path);
                    let marker = if has_manifest { " " } else { "?" };
                    let mode_str = match &s.mode {
                        Some(InstallMode::Link) => "[link]".to_string(),
                        Some(InstallMode::Copy) => "[copy]".to_string(),
                        None => String::new(),
                    };
                    println!("  {marker} {}  {mode_str}", s.name);
                }
                if skills
                    .iter()
                    .any(|s| !has_skill_manifest(&s.path))
                {
                    println!(
                        "\n  (?) No SKILL.md or CLAUDE.md found — these may not be valid skill repos."
                    );
                }
            }
            Ok(())
        }
        Command::Remove(args) => {
            skill::remove(&args.name)?;
            println!("Removed skill '{}'", args.name);
            Ok(())
        }
        Command::Update(args) => {
            let results = skill::update(args.name.as_deref())?;
            if results.is_empty() {
                println!("No skills to update.");
            }
            for (name, msg) in &results {
                println!("{name}: {msg}");
            }
            Ok(())
        }
        Command::Info(args) => {
            let s = skill::info(&args.name)?;
            println!("Skill: {}", s.name);
            println!("  Store:   {}", s.path.display());
            println!(
                "  Claude:  {}",
                config::claude_skills_dir().join(&s.name).display()
            );

            let deploy_status = match &s.mode {
                Some(InstallMode::Link) => "linked",
                Some(InstallMode::Copy) => "copied",
                None => "not deployed",
            };
            println!("  Status:  {deploy_status}");

            let skill_md = s.path.join("SKILL.md");
            let claude_md = s.path.join("CLAUDE.md");
            if skill_md.exists() {
                println!("  Manifest: SKILL.md");
            } else if claude_md.exists() {
                println!("  Manifest: CLAUDE.md");
            } else {
                println!("  Manifest: none found");
            }

            let entries = std::fs::read_dir(&s.path)
                .map_err(|e| format!("Cannot read skill directory: {e}"))?;
            let mut files = Vec::new();
            for entry in entries {
                let entry = entry.map_err(|e| format!("I/O error: {e}"))?;
                let fname = entry.file_name().to_string_lossy().to_string();
                if !fname.starts_with('.') && fname != "target" {
                    let ft = if entry.path().is_dir() { "/" } else { "" };
                    files.push(format!("{fname}{ft}"));
                }
            }
            if !files.is_empty() {
                files.sort();
                println!("  Files:");
                for f in &files {
                    println!("    - {f}");
                }
            }

            Ok(())
        }
    }
}

fn run_marketplace(cmd: MarketplaceCommand) -> Result<(), String> {
    match cmd {
        MarketplaceCommand::Add(args) => {
            marketplace::add(&args.url, args.name.as_deref())?;
            Ok(())
        }
        MarketplaceCommand::Remove(args) => {
            marketplace::remove(&args.name)?;
            println!("Removed marketplace '{}'", args.name);
            Ok(())
        }
        MarketplaceCommand::List => {
            let markets = marketplace::list()?;
            if markets.is_empty() {
                println!("No marketplaces configured.");
                println!(
                    "Use `ccsm marketplace add <url>` to add a skill marketplace."
                );
            } else {
                println!("Marketplaces ({})\n", markets.len());
                for (entry, count) in &markets {
                    let branch = entry.branch.as_deref().unwrap_or("?");
                    println!(
                        "  {}  ({} skills, branch: {})",
                        entry.name, count, branch
                    );
                    println!("    {}\n", entry.url);
                }
            }
            Ok(())
        }
        MarketplaceCommand::Update(args) => {
            let results = marketplace::update(args.name.as_deref())?;
            for (name, msg) in &results {
                println!("{name}: {msg}");
            }
            Ok(())
        }
    }
}

fn has_skill_manifest(path: &std::path::Path) -> bool {
    path.join("SKILL.md").exists()
        || path.join("skill.md").exists()
        || path.join("CLAUDE.md").exists()
        || path.join("claude.md").exists()
}
