# Changelog

## v0.3.0

### Added

- **Detailed `--version` output** — shows version, commit SHA, project name, and attribution (`by OnixByte`); version can be injected at build time via `CCSM_VERSION` env var
- **Marketplace creation guide** — README now documents how to create and publish a skill marketplace, including the `skills.json` format, skill entry fields, monorepo `path` support, and skill repository structure

## v0.2.0

### Added

- **Monorepo skill support** — skills can now live in subdirectories of a single repository via the `path` field in `skills.json`
- **Release-triggered CI** — workflow runs on `release: published` and uploads built artifacts directly to the release
- **Project documentation** — README, CONTRIBUTING, and LICENCE (MIT) files

### Fixed

- CI artifact upload compatibility with self-hosted Gitea (downgrade `upload-artifact` to v3)
- Cross-compilation for macOS and Windows targets via `cargo-zigbuild`

## v0.1.0

Initial release of Claude Code Skill Manager (CCSM) — a CLI tool for discovering, installing, and managing Claude Code skills from Git-based marketplace repositories.

### Features

- **Marketplace management** — register, list, update, and remove skill marketplaces from any Git repository exposing a `skills.json` manifest
- **Skill search** — browse and search available skills by name or description across all registered marketplaces
- **Skill installation** — download and deploy skills to `~/.claude/skills/` with copy or symlink modes
- **Skill lifecycle** — update installed skills to latest versions, remove unwanted skills, and inspect skill details
- **Cross-platform** — pre-built binaries for Linux, macOS, and Windows (amd64)

### Commands

| Command | Description |
|---|---|
| `ccsm marketplace add <url>` | Register a skill marketplace |
| `ccsm marketplace remove <name>` | Unregister a marketplace |
| `ccsm marketplace list` | List registered marketplaces |
| `ccsm marketplace update [name]` | Refresh marketplace manifests |
| `ccsm search [query]` | Search available skills |
| `ccsm install <name>` | Install a skill |
| `ccsm list` | Show installed skills |
| `ccsm info <name>` | Show skill details |
| `ccsm update [name]` | Update installed skills |
| `ccsm remove <name>` | Uninstall a skill |

### Installation

```bash
cargo install --path .
```

Or download a pre-built binary from the [Releases](https://onixbyte.dev/onixbyte/claude-code-skill-manager/releases) page.
