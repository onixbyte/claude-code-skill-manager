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

## Creating a skill marketplace

A marketplace is simply a public Git repository that contains a `skills.json` manifest at its root. CCSM reads this file to discover what skills are available.

### Repository structure

```
your-marketplace-repo/
  skills.json        ← manifest (required)
  README.md          ← optional landing page
```

### skills.json format

```json
{
  "name": "My Marketplace",
  "version": "1.0.0",
  "skills": [
    {
      "name": "my-skill",
      "description": "A short description shown in search results",
      "repository": "https://github.com/user/skill-repo",
      "branch": "main"
    },
    {
      "name": "another-skill",
      "description": "A skill that lives in a monorepo subdirectory",
      "repository": "https://github.com/user/monorepo",
      "branch": "main",
      "path": "skills/another-skill"
    }
  ]
}
```

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Marketplace display name |
| `version` | No | Arbitrary version string for your own tracking |
| `skills` | No | Array of skill entries (defaults to `[]`) |

**Skill entry fields:**

| Field | Required | Description |
|---|---|---|
| `name` | Yes | Unique skill identifier (used with `ccsm install <name>`) |
| `description` | Yes | One-line summary shown in `ccsm search` output |
| `repository` | Yes | Public Git repository URL for the skill |
| `branch` | No | Git branch to install from (defaults to `main` or `master`, auto-detected by the marketplace host) |
| `path` | No | Subdirectory within the repository where the skill files live — use for monorepos hosting multiple skills |

### What goes in a skill repository

Each skill repository (the one referenced by `repository`) is a standard Claude Code skill directory. At minimum it should contain one of:

- `SKILL.md` — the skill manifest (preferred)
- `CLAUDE.md` — alternative manifest filename
- Supporting files referenced by the manifest (scripts, templates, etc.)

See the [Claude Code skills documentation](https://docs.anthropic.com/en/docs/claude-code/skills) for details on writing skill manifests.

### Publishing your marketplace

1. Create a public Git repository with a `skills.json` following the format above.
2. Push it — make sure `skills.json` is at the repository root.
3. Anyone can register it with:

```bash
ccsm marketplace add https://github.com/your-username/your-marketplace
```

CCSM fetches `skills.json` from the raw `main` or `master` branch automatically, so both of those branch names work out of the box.

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
