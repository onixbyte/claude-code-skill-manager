# Claude Code Skill Manager (CCSM)

A CLI tool for discovering, installing, and managing [Claude Code](https://claude.ai/code) skills from Git-based marketplace repositories.

## Installation

### From source

```bash
git clone https://onixbyte.dev/onixbyte/claude-code-skill-manager.git
cd claude-code-skill-manager
cargo install --path .
```

### Pre-built binaries

Download the latest binary for your platform from the [Releases](https://onixbyte.dev/onixbyte/claude-code-skill-manager/releases) page.

## Quick start

```bash
# Register a skill marketplace
ccsm marketplace add https://github.com/example/skills

# Browse available skills
ccsm search

# Install a skill
ccsm install my-skill

# List installed skills
ccsm list

# See details about a skill
ccsm info my-skill

# Update installed skills
ccsm update

# Remove a skill
ccsm remove my-skill
```

## Commands

### Marketplace management

| Command | Description |
|---|---|
| `ccsm marketplace add <url>` | Register a Git repository as a skill marketplace |
| `ccsm marketplace remove <name>` | Unregister a marketplace |
| `ccsm marketplace list` | List registered marketplaces |
| `ccsm marketplace update [name]` | Refresh marketplace manifests |

### Skill management

| Command | Description |
|---|---|
| `ccsm search [query]` | Search available skills across all marketplaces |
| `ccsm install <name>` | Install a skill (supports `--copy` and `--link` modes) |
| `ccsm list` | Show installed skills |
| `ccsm info <name>` | Show details about an installed skill |
| `ccsm update [name]` | Update installed skills to latest |
| `ccsm remove <name>` | Uninstall a skill |

## How it works

Skills are directories placed under `~/.claude/skills/` that Claude Code reads at startup. CCSM manages these directories by pulling skill definitions from marketplace repositories that expose a `skills.json` manifest:

```json
{
  "name": "My Marketplace",
  "skills": [
    {
      "name": "example-skill",
      "description": "An example Claude Code skill",
      "repository": "https://github.com/user/skill-repo",
      "branch": "main"
    }
  ]
}
```

## Deploy modes

- **Copy** (default): Skill files are physically copied to `~/.claude/skills/<name>/`.
- **Link**: A symlink is created from `~/.claude/skills/<name>/` to the local store, useful for development.

## Building

```bash
cargo build --release
```

Cross-compilation targets: Linux (amd64), macOS (amd64), and Windows (amd64) via `cargo-zigbuild`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Licence

[MIT](LICENCE)
