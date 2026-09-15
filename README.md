# dotfiles

Managed with [chezmoi](https://chezmoi.io). Secrets encrypted with [age](https://age-encryption.org).

---

## Quick start on a new machine

```sh
bash <(curl -fsSL https://raw.githubusercontent.com/upe-tinkerly-up/dotfiles/main/bootstrap.sh)
```

This opens an interactive TUI checklist where you pick exactly which groups to install.
Everything already installed on the machine is skipped automatically.

---

## bootstrap.sh — full reference

### Modes

| Mode | Command |
|------|---------|
| Interactive TUI (default) | `bash bootstrap.sh` |
| Non-interactive, install all | `bash bootstrap.sh --non-interactive` |
| Non-interactive, selective | `bash bootstrap.sh --non-interactive --skip-hyprland --skip-fish` |
| Help | `bash bootstrap.sh --help` |

### Install groups

| Flag | Contents |
|------|----------|
| `--skip-fonts` | ttf-jetbrains-mono-nerd, noto-fonts, noto-fonts-emoji, ttf-nerd-fonts-symbols |
| `--skip-fish` | fish shell |
| `--skip-zsh` | zsh + plugins (autosuggestions, fast-syntax-highlighting, vi-mode, fzf-tab, history-substring-search) |
| `--skip-terminals` | alacritty, kitty, foot |
| `--skip-cli-tools` | fzf, ripgrep, fd, bat, eza, zoxide, direnv, lazygit, lazydocker, lf, btop, cava, fastfetch, starship |
| `--skip-hyprland` | hyprland, waybar, fuzzel, uwsm, wl-clipboard, xdg-user-dirs |
| `--skip-nodejs` | nvm + Node.js LTS + bun |
| `--skip-golang` | gvm + Go (latest stable) |
| `--skip-rust` | rustup + Rust stable + cargo |
| `--skip-neovim` | neovim (pacman) + python-pip + python-pynvim + headless Lazy sync + Mason LSP/tools |
| `--skip-dotfiles` | chezmoi init --apply from GitHub |

### What always installs (no prompt, no skip)

`git`, `curl`, `wget`, `base-devel`, `age`, `chezmoi`, `paru`
These are the minimum needed to run the rest.

### Neovim Mason packages (installed headlessly)

LSP servers: `ts_ls`, `html`, `cssls`, `tailwindcss`, `svelte`, `lua_ls`, `graphql`,
`emmet_ls`, `prismals`, `pyright`, `gopls`, `marksman`, `markdownlint-cli2`

Tools: `prettier`, `stylua`, `isort`, `black`, `pylint`, `gofumpt`, `goimports`,
`golangci-lint`, `delve`

> **Note:** Mason needs Node.js (ts_ls, html, css, prettier), Go (gopls, gofumpt, goimports,
> golangci-lint, delve), and Python/pip (isort, black, pylint). If you skip those language
> groups and they aren't already installed, bootstrap will warn you before proceeding.

### Node.js version

nvm installs the **current LTS** at time of run (`nvm install --lts`).
The default alias is set to `lts/*` so `nvm use default` always gives LTS.
nvm is installed to `~/.config/nvm` (matches existing setup).

### Go version

gvm installs the **latest stable** fetched live from `https://go.dev/dl/?mode=json`.
Falls back to binary install first; compiles from source only if binary is unavailable.
Falls back to `pacman go` if gvm itself fails to install.

### Rust

rustup installs `stable` toolchain + `rust-src` + `rust-analyzer` components.
cargo env is sourced from `~/.cargo/env`.

### zsh plugins

Plugins are **not cloned by bootstrap** — they are managed by `_zplugin_load()` inside
`dot_config/zsh/plugins.zsh`. On the first interactive zsh session they clone themselves
from GitHub automatically. To update later: run `zplugin-update` in zsh.

Plugins managed this way:
- zsh-users/zsh-autosuggestions
- zsh-users/zsh-history-substring-search
- jeffreytse/zsh-vi-mode
- Aloxaf/fzf-tab
- zdharma-continuum/fast-syntax-highlighting

### Age key (encrypted secrets)

Secrets are encrypted with age. The key must exist at `~/.config/chezmoi/key.txt`
**before** `chezmoi apply` runs, otherwise encrypted files are silently skipped.

Bootstrap handles this in three ways:
1. Key already exists → proceeds automatically
2. Interactive: prompts to paste key content or provide a backup file path
3. Non-interactive without key → warns and continues (apply may be partial)

To apply dotfiles manually after placing the key:
```sh
chezmoi apply
```

### Log file

All output is appended to `~/bootstrap.log`. If anything fails, check there first.
The script is **idempotent** — safe to re-run at any time.

---

## Manual chezmoi usage

Apply latest changes from repo:
```sh
chezmoi update
```

Check what would change without applying:
```sh
chezmoi diff
```

Edit a managed file:
```sh
chezmoi edit ~/.config/fish/config.fish
```

Add a new file to management:
```sh
chezmoi add ~/.config/somefile
```

Commit + push changes:
```sh
chezmoi cd
git add -A && git commit -m "update" && git push
```

---

## Contents

| Category | Details |
|----------|---------|
| Shell | bash, zsh, fish |
| zsh plugins | fzf-tab, autosuggestions, fast-syntax-highlighting, vi-mode, history-substring-search |
| Editor | neovim (custom config, mason LSPs) |
| Terminals | alacritty, kitty, foot |
| WM | hyprland |
| Prompt | starship |
| File manager | lf |
| Tools | btop, cava, fastfetch, lazygit, lazydocker |
| Runtimes | Node.js (nvm), Go (gvm), Rust (rustup) |
| Encryption | age (chezmoi secrets) |

---

## Requirements

- Arch-based distro (uses `pacman` + `paru`)
- `sudo` access
- Internet connection
- age key at `~/.config/chezmoi/key.txt` for encrypted files
