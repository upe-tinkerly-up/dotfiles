#!/usr/bin/env bash
# =============================================================================
# bootstrap.sh — Fresh machine setup for upe-tinkerly-up dotfiles
# =============================================================================
#
# USAGE
#   Interactive (TUI checklist):
#     bash bootstrap.sh
#
#   Non-interactive (install all, respect --skip-* flags):
#     bash bootstrap.sh --non-interactive
#
#   Skip specific groups:
#     bash bootstrap.sh --skip-fish --skip-hyprland
#
#   Run remotely (after pushing to GitHub):
#     bash <(curl -fsSL https://raw.githubusercontent.com/upe-tinkerly-up/dotfiles/main/bootstrap.sh)
#
#   Show help:
#     bash bootstrap.sh --help
#
# =============================================================================
# GROUPS
#   fonts      — JetBrainsMono Nerd Font, Noto Fonts, ttf-nerd-fonts-symbols
#   fish       — Fish shell
#   zsh        — Zsh + plugins (autosuggestions, syntax-hl, vi-mode, fzf-tab,
#                  history-substring-search). Plugins are auto-cloned on first
#                  zsh launch via the _zplugin_load() helper in plugins.zsh.
#   terminals  — Alacritty, Kitty, Foot
#   cli        — fzf, ripgrep, fd, bat, eza, zoxide, direnv, lazygit,
#                  lazydocker, lf, btop, cava, fastfetch, starship
#   hyprland   — Hyprland, waybar, fuzzel, uwsm, wl-clipboard, xdg-user-dirs
#   nodejs     — nvm + Node.js LTS + bun
#   golang     — gvm + Go stable (latest)
#   rust       — rustup + Rust stable + cargo
#   neovim     — Neovim (pacman) + python-pip + headless Lazy sync +
#                  Mason LSPs & tools. Requires node (ts_ls, prettier…),
#                  go (gopls, gofumpt…), and optionally rust (rust-analyzer).
#   dotfiles   — chezmoi init + apply from GitHub. Requires age key at
#                  ~/.config/chezmoi/key.txt to decrypt encrypted secrets.
#
# =============================================================================
# AGE KEY
#   Secrets in this dotfiles repo are encrypted with age. Before chezmoi apply,
#   the key must exist at ~/.config/chezmoi/key.txt.
#   If missing, the script will prompt you to paste the key content.
#
# =============================================================================
# NEOVIM MASON PACKAGES (auto-installed headlessly)
#   LSP servers : ts_ls, html, cssls, tailwindcss, svelte, lua_ls, graphql,
#                 emmet_ls, prismals, pyright, gopls, marksman, markdownlint-cli2
#   Tools       : prettier, stylua, isort, black, pylint, gofumpt, goimports,
#                 golangci-lint, delve
#
# =============================================================================
# NOTES
#   - Idempotent: safe to re-run. Every tool is checked before installing.
#   - Arch-based only (uses pacman + paru).
#   - All output is appended to ~/bootstrap.log.
#   - nvm is installed to ~/.config/nvm (matching existing setup).
#   - gvm is installed to ~/.gvm.
#   - rustup installs to ~/.rustup / ~/.cargo.
# =============================================================================

set -euo pipefail
IFS=$'\n\t'

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------
DOTFILES_USER="upe-tinkerly-up"
NVM_DIR="${NVM_DIR:-$HOME/.config/nvm}"
NVM_INSTALL_VERSION="v0.40.3"
LOG_FILE="$HOME/bootstrap.log"

# ---------------------------------------------------------------------------
# Colors
# ---------------------------------------------------------------------------
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
BOLD='\033[1m'
DIM='\033[2m'
NC='\033[0m'

# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------
_ts() { date '+%H:%M:%S'; }
log_file_init() {
  mkdir -p "$(dirname "$LOG_FILE")"
  echo "=============================================" >> "$LOG_FILE"
  echo "Bootstrap started: $(date)" >> "$LOG_FILE"
  echo "=============================================" >> "$LOG_FILE"
}

log()  { echo -e "${GREEN}[OK]${NC}  $*" | tee -a "$LOG_FILE"; }
info() { echo -e "${BLUE}[..]${NC}  $*" | tee -a "$LOG_FILE"; }
warn() { echo -e "${YELLOW}[!!]${NC}  $*" | tee -a "$LOG_FILE"; }
err()  { echo -e "${RED}[EE]${NC}  $*" | tee -a "$LOG_FILE"; }
step() { echo -e "\n${BOLD}${CYAN}===>${NC}${BOLD} $*${NC}" | tee -a "$LOG_FILE"; }
die()  { err "$*"; exit 1; }

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
has() { command -v "$1" &>/dev/null; }

pkg_installed() { pacman -Q "$1" &>/dev/null 2>&1; }

# Returns 0 (true) if the binary OR the package is already present
is_installed() {
  local bin="$1"
  local pkg="${2:-$1}"
  has "$bin" || pkg_installed "$pkg"
}

# Install via pacman (idempotent via --needed)
pacman_install() {
  info "pacman: installing $*"
  sudo pacman -S --needed --noconfirm "$@" 2>&1 | tee -a "$LOG_FILE" \
    && log "pacman: $* installed" \
    || die "pacman: failed to install $*"
}

# Install via paru (idempotent via --needed)
paru_install() {
  info "paru: installing $*"
  paru -S --needed --noconfirm "$@" 2>&1 | tee -a "$LOG_FILE" \
    && log "paru: $* installed" \
    || die "paru: failed to install $*"
}

# ---------------------------------------------------------------------------
# Counters for final summary
# ---------------------------------------------------------------------------
INSTALLED_GROUPS=()
SKIPPED_GROUPS=()
FAILED_GROUPS=()

mark_installed() { INSTALLED_GROUPS+=("$1"); }
mark_skipped()   { SKIPPED_GROUPS+=("$1"); }
mark_failed()    { FAILED_GROUPS+=("$1"); warn "Group $1 had errors — check $LOG_FILE"; }

# ---------------------------------------------------------------------------
# Parse CLI flags
# ---------------------------------------------------------------------------
NON_INTERACTIVE=false
SKIP_FONTS=false
SKIP_FISH=false
SKIP_ZSH=false
SKIP_TERMINALS=false
SKIP_CLI_TOOLS=false
SKIP_HYPRLAND=false
SKIP_NODEJS=false
SKIP_GOLANG=false
SKIP_RUST=false
SKIP_NEOVIM=false
SKIP_TMUX=false
SKIP_DOTFILES=false
SKIP_VAULT=false

print_help() {
  cat <<'EOF'
Usage: bash bootstrap.sh [OPTIONS]

Options:
  --non-interactive    Skip TUI, install all groups (respects --skip-* flags)
  --skip-fonts         Skip font installation
  --skip-fish          Skip fish shell
  --skip-zsh           Skip zsh + plugins
  --skip-terminals     Skip alacritty / kitty / foot
  --skip-cli-tools     Skip CLI tools (fzf, ripgrep, bat, eza, lazygit, etc.)
  --skip-hyprland      Skip Hyprland WM + waybar + fuzzel + uwsm
  --skip-nodejs        Skip nvm + Node.js LTS + bun
  --skip-golang        Skip gvm + Go (stable)
  --skip-rust          Skip rustup + Rust stable
  --skip-neovim        Skip neovim + Mason LSP/tools headless install
  --skip-tmux          Skip tmux + TPM plugins (vim-tmux-navigator, resurrect, continuum)
  --skip-dotfiles      Skip chezmoi init + apply
  --skip-vault         Skip vault CLI build + rclone setup
  -h, --help           Show this message
EOF
}

for arg in "$@"; do
  case "$arg" in
    --non-interactive) NON_INTERACTIVE=true ;;
    --skip-fonts)      SKIP_FONTS=true ;;
    --skip-fish)       SKIP_FISH=true ;;
    --skip-zsh)        SKIP_ZSH=true ;;
    --skip-terminals)  SKIP_TERMINALS=true ;;
    --skip-cli-tools)  SKIP_CLI_TOOLS=true ;;
    --skip-hyprland)   SKIP_HYPRLAND=true ;;
    --skip-nodejs)     SKIP_NODEJS=true ;;
    --skip-golang)     SKIP_GOLANG=true ;;
    --skip-rust)       SKIP_RUST=true ;;
    --skip-neovim)     SKIP_NEOVIM=true ;;
    --skip-tmux)       SKIP_TMUX=true ;;
    --skip-dotfiles)   SKIP_DOTFILES=true ;;
    --skip-vault)      SKIP_VAULT=true ;;
    -h|--help)         print_help; exit 0 ;;
    *) die "Unknown flag: $arg  — run with --help for usage." ;;
  esac
done

# ---------------------------------------------------------------------------
# Sanity checks
# ---------------------------------------------------------------------------
[[ $EUID -eq 0 ]] && die "Do not run as root. The script uses sudo internally."

# Detect WSL
IS_WSL=false
if grep -qi microsoft /proc/version &>/dev/null 2>&1; then
  IS_WSL=true
  warn "WSL detected — some paths/tools may differ from native Linux"
fi

if ! has pacman; then
  die "pacman not found. This script is for Arch-based systems only."
fi

log_file_init
step "Bootstrap: upe-tinkerly-up dotfiles"
info "Log: $LOG_FILE"
info "Date: $(date)"

# ---------------------------------------------------------------------------
# Step 1: Base prerequisites (always installed, no prompt)
# ---------------------------------------------------------------------------
step "Step 1/9 — Base prerequisites"
info "Installing: git curl wget base-devel age chezmoi paru"

# base-devel needed for AUR builds (makepkg)
for pkg in git curl wget base-devel; do
  if pkg_installed "$pkg"; then
    info "$pkg — already installed, skip"
  else
    pacman_install "$pkg"
  fi
done

# paru — AUR helper (CachyOS has it in repo)
if has paru; then
  info "paru — already installed, skip"
elif pkg_installed paru; then
  info "paru (package) — already installed, skip"
else
  info "Installing paru..."
  if sudo pacman -S --needed --noconfirm paru 2>&1 | tee -a "$LOG_FILE"; then
    log "paru installed via pacman"
  else
    info "paru not in pacman repos, building from AUR..."
    _TMPDIR=$(mktemp -d)
    git clone --depth=1 https://aur.archlinux.org/paru-bin.git "$_TMPDIR/paru-bin" >> "$LOG_FILE" 2>&1
    (cd "$_TMPDIR/paru-bin" && makepkg -si --noconfirm >> "$LOG_FILE" 2>&1)
    rm -rf "$_TMPDIR"
    log "paru built and installed"
  fi
fi

# age — for chezmoi secret decryption
if is_installed age; then
  info "age — already installed, skip"
else
  pacman_install age
fi

# chezmoi — dotfile manager
if is_installed chezmoi; then
  info "chezmoi — already installed, skip"
else
  # Try pacman first, fallback to official installer
  if sudo pacman -S --needed --noconfirm chezmoi 2>&1 | tee -a "$LOG_FILE"; then
    log "chezmoi installed via pacman"
  else
    info "chezmoi not in pacman, using official installer..."
    sh -c "$(curl -fsLS get.chezmoi.io)" -- -b "$HOME/bin" >> "$LOG_FILE" 2>&1
    log "chezmoi installed to ~/bin"
    export PATH="$HOME/bin:$PATH"
  fi
fi

# libnewt (provides whiptail) — for TUI menu
if ! has whiptail; then
  sudo pacman -S --needed --noconfirm libnewt 2>&1 | tee -a "$LOG_FILE" || true
fi

# ---------------------------------------------------------------------------
# Step 2: Interactive group selection
# ---------------------------------------------------------------------------
step "Step 2/9 — Group selection"

# Helper: returns " [installed]" or ""
_status() {
  local bin="$1"; local pkg="${2:-$1}"
  is_installed "$bin" "$pkg" && echo " [installed]" || echo ""
}

if [[ "$NON_INTERACTIVE" == false ]]; then

  if has whiptail; then
    # Pre-compute status labels (avoids subshell quoting issues in whiptail)
    L_FONTS="Fonts: JetBrainsMono NF, Noto, Nerd Symbols$(_status ttf-jetbrains-mono-nerd)"
    L_FISH="Fish shell$(_status fish)"
    L_ZSH="Zsh + plugins (autosuggestions, syntax-hl, vi-mode)$(_status zsh)"
    L_TERM="Terminals: Alacritty, Kitty, Foot$(_status alacritty)"
    L_CLI="CLI: fzf ripgrep bat eza lazygit lazydocker btop...$(_status lazygit)"
    L_HYPR="Hyprland + waybar + fuzzel + uwsm + wl-clipboard$(_status hyprland)"
    L_NODE="nvm + Node.js LTS + bun$(_status node)"
    L_GO="gvm + Go (stable, latest)$(_status go)"
    L_RUST="rustup + Rust stable + cargo$(_status rustc)"
    L_NVIM="Neovim + Mason LSPs + tools (needs node/go)$(_status nvim neovim)"
    L_TMUX="tmux + TPM plugins (vim-tmux-navigator, resurrect, continuum)$(_status tmux)"
    L_DOTS="chezmoi init + apply from GitHub (needs age key)"
    L_VAULT="vault CLI build + rclone (Obsidian sync)$(_status vault)"

    CHOICES=$(whiptail \
      --title "bootstrap.sh — Select groups to install" \
      --checklist \
      "Space = toggle  |  Enter = confirm\nItems marked [installed] will be skipped automatically." \
      28 74 13 \
      "fonts"     "$L_FONTS"    ON \
      "fish"      "$L_FISH"     ON \
      "zsh"       "$L_ZSH"      ON \
      "terminals" "$L_TERM"     ON \
      "cli"       "$L_CLI"      ON \
      "hyprland"  "$L_HYPR"     ON \
      "nodejs"    "$L_NODE"     ON \
      "golang"    "$L_GO"       ON \
      "rust"      "$L_RUST"     ON \
      "neovim"    "$L_NVIM"     ON \
      "tmux"      "$L_TMUX"     ON \
      "dotfiles"  "$L_DOTS"     ON \
      "vault"     "$L_VAULT"    ON \
      3>&1 1>&2 2>&3) || { info "Cancelled."; exit 0; }

    [[ "$CHOICES" != *'"fonts"'*     ]] && SKIP_FONTS=true
    [[ "$CHOICES" != *'"fish"'*      ]] && SKIP_FISH=true
    [[ "$CHOICES" != *'"zsh"'*       ]] && SKIP_ZSH=true
    [[ "$CHOICES" != *'"terminals"'* ]] && SKIP_TERMINALS=true
    [[ "$CHOICES" != *'"cli"'*       ]] && SKIP_CLI_TOOLS=true
    [[ "$CHOICES" != *'"hyprland"'*  ]] && SKIP_HYPRLAND=true
    [[ "$CHOICES" != *'"nodejs"'*    ]] && SKIP_NODEJS=true
    [[ "$CHOICES" != *'"golang"'*    ]] && SKIP_GOLANG=true
    [[ "$CHOICES" != *'"rust"'*      ]] && SKIP_RUST=true
    [[ "$CHOICES" != *'"neovim"'*    ]] && SKIP_NEOVIM=true
    [[ "$CHOICES" != *'"tmux"'*      ]] && SKIP_TMUX=true
    [[ "$CHOICES" != *'"dotfiles"'*  ]] && SKIP_DOTFILES=true
    [[ "$CHOICES" != *'"cli"'*       ]] && SKIP_CLI_TOOLS=true
    [[ "$CHOICES" != *'"hyprland"'*  ]] && SKIP_HYPRLAND=true
    [[ "$CHOICES" != *'"nodejs"'*    ]] && SKIP_NODEJS=true
    [[ "$CHOICES" != *'"golang"'*    ]] && SKIP_GOLANG=true
    [[ "$CHOICES" != *'"rust"'*      ]] && SKIP_RUST=true
    [[ "$CHOICES" != *'"neovim"'*    ]] && SKIP_NEOVIM=true
    [[ "$CHOICES" != *'"dotfiles"'*  ]] && SKIP_DOTFILES=true

  else
    # Fallback: numbered text menu
    warn "whiptail not available — using text menu"
    echo ""
    printf "  %-3s %-12s %s\n" "#" "Group" "Status"
    printf "  %-3s %-12s %s\n" "---" "------------" "--------"
    printf "  %-3s %-12s %s\n" "1"  "fonts"        "$(_status ttf-jetbrains-mono-nerd)"
    printf "  %-3s %-12s %s\n" "2"  "fish"         "$(_status fish)"
    printf "  %-3s %-12s %s\n" "3"  "zsh"          "$(_status zsh)"
    printf "  %-3s %-12s %s\n" "4"  "terminals"    "$(_status alacritty)"
    printf "  %-3s %-12s %s\n" "5"  "cli"          "$(_status lazygit)"
    printf "  %-3s %-12s %s\n" "6"  "hyprland"     "$(_status hyprland)"
    printf "  %-3s %-12s %s\n" "7"  "nodejs"       "$(_status node)"
    printf "  %-3s %-12s %s\n" "8"  "golang"       "$(_status go)"
    printf "  %-3s %-12s %s\n" "9"  "rust"         "$(_status rustc)"
    printf "  %-3s %-12s %s\n" "10" "neovim"       "$(_status nvim neovim)"
    printf "  %-3s %-12s %s\n" "11" "dotfiles"     "(chezmoi)"
    printf "  %-3s %-12s %s\n" "12" "vault"        "(vault-cli + rclone)"
    echo ""
    echo "  Enter numbers separated by spaces, or press Enter for ALL."
    read -rp "  Your choice: " RAW_CHOICE

    if [[ -n "$RAW_CHOICE" ]]; then
      # Deselect all first
      SKIP_FONTS=true; SKIP_FISH=true; SKIP_ZSH=true; SKIP_TERMINALS=true
      SKIP_CLI_TOOLS=true; SKIP_HYPRLAND=true; SKIP_NODEJS=true
      SKIP_GOLANG=true; SKIP_RUST=true; SKIP_NEOVIM=true; SKIP_DOTFILES=true; SKIP_VAULT=true
      for n in $RAW_CHOICE; do
        case "$n" in
          1)  SKIP_FONTS=false ;;
          2)  SKIP_FISH=false ;;
          3)  SKIP_ZSH=false ;;
          4)  SKIP_TERMINALS=false ;;
          5)  SKIP_CLI_TOOLS=false ;;
          6)  SKIP_HYPRLAND=false ;;
          7)  SKIP_NODEJS=false ;;
          8)  SKIP_GOLANG=false ;;
          9)  SKIP_RUST=false ;;
          10) SKIP_NEOVIM=false ;;
          11) SKIP_DOTFILES=false ;;
          12) SKIP_VAULT=false ;;
          *) warn "Unknown number: $n — ignored" ;;
        esac
      done
    fi
  fi
fi

# Dependency warnings
if [[ "$SKIP_NEOVIM" == false ]]; then
  [[ "$SKIP_NODEJS" == true ]] && ! has node && \
    warn "Neovim: Node.js not selected and not installed — ts_ls, html, css LSPs need node/npm"
  [[ "$SKIP_GOLANG" == true ]] && ! has go && \
    warn "Neovim: Go not selected and not installed — gopls, gofumpt, goimports, delve need go"
  [[ "$SKIP_RUST" == true ]] && ! has rustc && \
    warn "Neovim: Rust not selected and not installed — rust_analyzer needs cargo"
fi

# Confirm plan
step "Install plan"
echo ""
_plan() {
  local name="$1"; local skip="$2"
  if [[ "$skip" == true ]]; then
    echo -e "  ${DIM}SKIP${NC}     $name"
  else
    echo -e "  ${GREEN}INSTALL${NC}  $name"
  fi
}
_plan "fonts"     "$SKIP_FONTS"
_plan "fish"      "$SKIP_FISH"
_plan "zsh"       "$SKIP_ZSH"
_plan "terminals" "$SKIP_TERMINALS"
_plan "cli-tools" "$SKIP_CLI_TOOLS"
_plan "hyprland"  "$SKIP_HYPRLAND"
_plan "nodejs"    "$SKIP_NODEJS"
_plan "golang"    "$SKIP_GOLANG"
_plan "rust"       "$SKIP_RUST"
_plan "neovim"     "$SKIP_NEOVIM"
_plan "tmux"       "$SKIP_TMUX"
_plan "dotfiles"   "$SKIP_DOTFILES"
_plan "vault"      "$SKIP_VAULT"
echo ""

if [[ "$NON_INTERACTIVE" == false ]]; then
  read -rp "  Proceed? [Y/n] " CONFIRM
  [[ "${CONFIRM:-Y}" =~ ^[Nn] ]] && { info "Aborted."; exit 0; }
fi

# ---------------------------------------------------------------------------
# Step 3: Fonts
# ---------------------------------------------------------------------------
step "Step 3/9 — Fonts"

if [[ "$SKIP_FONTS" == true ]]; then
  info "fonts — skipped"; mark_skipped "fonts"
else
  FONT_PKGS=(
    ttf-jetbrains-mono-nerd   # used in alacritty, kitty, foot configs
    noto-fonts                # broad unicode coverage
    noto-fonts-emoji          # emoji
    ttf-nerd-fonts-symbols    # icons for neovim lualine, nvim-tree, etc.
  )
  FONT_CHANGED=false
  for fp in "${FONT_PKGS[@]}"; do
    if pkg_installed "$fp"; then
      info "$fp — already installed, skip"
    else
      pacman_install "$fp"
      FONT_CHANGED=true
    fi
  done
  # Rebuild font cache if anything new was installed
  [[ "$FONT_CHANGED" == true ]] && { info "Rebuilding font cache..."; fc-cache -fv >> "$LOG_FILE" 2>&1; }
  log "fonts done"; mark_installed "fonts"
fi

# ---------------------------------------------------------------------------
# Step 4: Shell — Fish
# ---------------------------------------------------------------------------
step "Step 4/9 — Shell"

if [[ "$SKIP_FISH" == true ]]; then
  info "fish — skipped"; mark_skipped "fish"
else
  if is_installed fish; then
    info "fish — already installed, skip"
  else
    pacman_install fish
  fi
  log "fish done"; mark_installed "fish"
fi

if [[ "$SKIP_ZSH" == true ]]; then
  info "zsh — skipped"; mark_skipped "zsh"
else
  if is_installed zsh; then
    info "zsh — already installed, skip"
  else
    pacman_install zsh
  fi
  # zsh plugins (fzf-tab, autosuggestions, syntax-hl, vi-mode,
  # history-substring-search) are managed by _zplugin_load() inside
  # plugins.zsh — they are cloned automatically on the first interactive
  # zsh session. No manual cloning needed here.
  info "zsh plugins will auto-install on first zsh launch (via _zplugin_load)"
  log "zsh done"; mark_installed "zsh"
fi

# ---------------------------------------------------------------------------
# Step 5: Terminals
# ---------------------------------------------------------------------------
step "Step 5/9 — Terminals"

if [[ "$SKIP_TERMINALS" == true ]]; then
  info "terminals — skipped"; mark_skipped "terminals"
else
  for t in alacritty kitty foot; do
    if is_installed "$t"; then
      info "$t — already installed, skip"
    else
      pacman_install "$t"
    fi
  done
  log "terminals done"; mark_installed "terminals"
fi

# ---------------------------------------------------------------------------
# Step 6: CLI Tools
# ---------------------------------------------------------------------------
step "Step 6/9 — CLI Tools"

if [[ "$SKIP_CLI_TOOLS" == true ]]; then
  info "cli-tools — skipped"; mark_skipped "cli-tools"
else
  # Map: binary => package name (when they differ)
  declare -A CLI_PKGS=(
    [fzf]=fzf
    [rg]=ripgrep
    [fd]=fd
    [bat]=bat
    [eza]=eza
    [zoxide]=zoxide
    [direnv]=direnv
    [lazygit]=lazygit
    [lazydocker]=lazydocker
    [lf]=lf
    [btop]=btop
    [cava]=cava
    [fastfetch]=fastfetch
    [starship]=starship
  )

  for bin in "${!CLI_PKGS[@]}"; do
    pkg="${CLI_PKGS[$bin]}"
    if is_installed "$bin" "$pkg"; then
      info "$pkg — already installed, skip"
    else
      # Try pacman first, paru as fallback for AUR packages
      if pacman -Si "$pkg" &>/dev/null 2>&1; then
        pacman_install "$pkg"
      else
        paru_install "$pkg"
      fi
    fi
  done
  log "cli-tools done"; mark_installed "cli-tools"
fi

# ---------------------------------------------------------------------------
# Step 7: Hyprland WM
# ---------------------------------------------------------------------------
step "Step 7/9 — Hyprland WM"

if [[ "$SKIP_HYPRLAND" == true ]]; then
  info "hyprland — skipped"; mark_skipped "hyprland"
else
  declare -A HYPR_PKGS=(
    [hyprland]=hyprland
    [waybar]=waybar
    [fuzzel]=fuzzel
    [uwsm]=uwsm
    [wl-copy]=wl-clipboard
    [xdg-user-dirs]=xdg-user-dirs
  )

  for bin in "${!HYPR_PKGS[@]}"; do
    pkg="${HYPR_PKGS[$bin]}"
    if is_installed "$bin" "$pkg"; then
      info "$pkg — already installed, skip"
    else
      if pacman -Si "$pkg" &>/dev/null 2>&1; then
        pacman_install "$pkg"
      else
        paru_install "$pkg"
      fi
    fi
  done

  # Initialize xdg user dirs (~/, ~/Documents, etc.) if not set
  if has xdg-user-dirs-update; then
    xdg-user-dirs-update >> "$LOG_FILE" 2>&1 || true
  fi

  log "hyprland done"; mark_installed "hyprland"
fi

# ---------------------------------------------------------------------------
# Step 8a: Node.js via nvm
# ---------------------------------------------------------------------------
step "Step 8/9 — Language runtimes"

if [[ "$SKIP_NODEJS" == true ]]; then
  info "nodejs — skipped"; mark_skipped "nodejs"
else
  if has node && has npm; then
    info "node $(node --version) — already installed, skip"
    info "npm $(npm --version) — already installed, skip"
  else
    info "Installing nvm $NVM_INSTALL_VERSION..."
    export NVM_DIR="$NVM_DIR"
    mkdir -p "$NVM_DIR"
    # Download and run nvm installer with NVM_DIR overridden
    INSTALL_SCRIPT=$(mktemp)
    curl -fsSL "https://raw.githubusercontent.com/nvm-sh/nvm/$NVM_INSTALL_VERSION/install.sh" \
      -o "$INSTALL_SCRIPT" >> "$LOG_FILE" 2>&1
    NVM_DIR="$NVM_DIR" bash "$INSTALL_SCRIPT" >> "$LOG_FILE" 2>&1
    rm -f "$INSTALL_SCRIPT"

    # Source nvm in this session
    # shellcheck source=/dev/null
    source "$NVM_DIR/nvm.sh"

    info "Installing Node.js LTS..."
    nvm install --lts >> "$LOG_FILE" 2>&1
    nvm use --lts >> "$LOG_FILE" 2>&1
    nvm alias default 'lts/*' >> "$LOG_FILE" 2>&1
    log "Node.js $(node --version) installed"
  fi

  # bun
  if is_installed bun; then
    info "bun — already installed, skip"
  else
    info "Installing bun..."
    if pkg_installed bun; then
      info "bun (package) already installed, skip"
    elif pacman -Si bun &>/dev/null 2>&1; then
      pacman_install bun
    else
      paru_install bun-bin 2>/dev/null || \
        { curl -fsSL https://bun.sh/install | bash >> "$LOG_FILE" 2>&1 && log "bun installed via official installer"; }
    fi
  fi

  log "nodejs done"; mark_installed "nodejs"
fi

# ---------------------------------------------------------------------------
# Step 8b: Go via gvm
# ---------------------------------------------------------------------------
if [[ "$SKIP_GOLANG" == true ]]; then
  info "golang — skipped"; mark_skipped "golang"
else
  if has go; then
    info "go $(go version) — already installed, skip"
  else
    info "Fetching latest stable Go version..."
    GO_VERSION=$(curl -fsSL "https://go.dev/dl/?mode=json" \
      | python3 -c "import sys,json; d=json.load(sys.stdin); print(d[0]['version'])" 2>/dev/null \
      || echo "go1.27.1")
    info "Target Go version: $GO_VERSION"

    if has gvm; then
      info "gvm already installed"
    else
      info "Installing gvm..."
      # gvm needs: bison, gcc, make (via base-devel, already installed)
      GVM_SCRIPT=$(mktemp)
      curl -fsSL https://raw.githubusercontent.com/moovweb/gvm/master/binscripts/gvm-installer \
        -o "$GVM_SCRIPT" >> "$LOG_FILE" 2>&1
      bash "$GVM_SCRIPT" >> "$LOG_FILE" 2>&1
      rm -f "$GVM_SCRIPT"
    fi

    # Source gvm in this session
    # shellcheck source=/dev/null
    [[ -s "$HOME/.gvm/scripts/gvm" ]] && source "$HOME/.gvm/scripts/gvm"

    if has gvm; then
      info "Installing $GO_VERSION (binary build)..."
      gvm install "$GO_VERSION" --binary >> "$LOG_FILE" 2>&1 \
        || { warn "$GO_VERSION binary not found, compiling from source (slower)..."; \
             gvm install "$GO_VERSION" >> "$LOG_FILE" 2>&1; }
      gvm use "$GO_VERSION" --default >> "$LOG_FILE" 2>&1
      log "go $(go version) installed"
    else
      warn "gvm failed to install. Falling back to pacman go..."
      pacman_install go
    fi
  fi

  log "golang done"; mark_installed "golang"
fi

# ---------------------------------------------------------------------------
# Step 8c: Rust via rustup
# ---------------------------------------------------------------------------
if [[ "$SKIP_RUST" == true ]]; then
  info "rust — skipped"; mark_skipped "rust"
else
  if has rustc && has cargo; then
    info "rustc $(rustc --version) — already installed, skip"
  else
    if is_installed rustup; then
      info "rustup already installed"
    else
      info "Installing rustup..."
      RUSTUP_SCRIPT=$(mktemp)
      curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs -o "$RUSTUP_SCRIPT" >> "$LOG_FILE" 2>&1
      sh "$RUSTUP_SCRIPT" -y --no-modify-path >> "$LOG_FILE" 2>&1
      rm -f "$RUSTUP_SCRIPT"
    fi

    # Source cargo env in this session
    # shellcheck source=/dev/null
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env"

    info "Setting stable toolchain as default..."
    rustup default stable >> "$LOG_FILE" 2>&1
    rustup component add rust-src rust-analyzer >> "$LOG_FILE" 2>&1 || \
      warn "rust-src/rust-analyzer component failed — non-fatal"

    log "rustc $(rustc --version) installed"
  fi

  log "rust done"; mark_installed "rust"
fi

# ---------------------------------------------------------------------------
# Step 9a: Dotfiles via chezmoi (age key + init + apply)
# ---------------------------------------------------------------------------
step "Step 9/9 — Dotfiles + Neovim"

if [[ "$SKIP_DOTFILES" == true ]]; then
  info "dotfiles — skipped"; mark_skipped "dotfiles"
else
  # ---- Age key setup ----
  AGE_KEY_PATH="$HOME/.config/chezmoi/key.txt"
  mkdir -p "$(dirname "$AGE_KEY_PATH")"

  if [[ -f "$AGE_KEY_PATH" ]]; then
    info "age key found at $AGE_KEY_PATH — ok"
  else
    warn "age key not found at $AGE_KEY_PATH"
    echo ""
    echo "  Your dotfiles contain encrypted secrets (age)."
    echo "  Without the key, chezmoi cannot decrypt them."
    echo ""
    echo "  Options:"
    echo "    1) Paste the key content now"
    echo "    2) Provide the path to the backup key file"
    echo "    3) Skip (chezmoi apply may fail for encrypted files)"
    echo ""
    read -rp "  Choice [1/2/3]: " KEY_CHOICE

    case "${KEY_CHOICE:-3}" in
      1)
        echo "  Paste the key content below, then press Enter + Ctrl-D:"
        KEY_CONTENT=$(cat)
        echo "$KEY_CONTENT" > "$AGE_KEY_PATH"
        chmod 600 "$AGE_KEY_PATH"
        log "age key written to $AGE_KEY_PATH"
        ;;
      2)
        read -rp "  Path to key file: " KEY_FILE_PATH
        KEY_FILE_PATH="${KEY_FILE_PATH/#\~/$HOME}"  # expand tilde
        if [[ -f "$KEY_FILE_PATH" ]]; then
          cp "$KEY_FILE_PATH" "$AGE_KEY_PATH"
          chmod 600 "$AGE_KEY_PATH"
          log "age key copied to $AGE_KEY_PATH"
        else
          warn "File not found: $KEY_FILE_PATH — chezmoi may fail for encrypted files"
        fi
        ;;
      3)
        warn "age key skipped — encrypted files will not be decrypted"
        ;;
    esac
  fi

  # ---- chezmoi init + apply ----
  CHEZMOI_SOURCE_DIR="$HOME/.local/share/chezmoi"

  if [[ -d "$CHEZMOI_SOURCE_DIR/.git" ]]; then
    info "chezmoi source already exists at $CHEZMOI_SOURCE_DIR"
    info "Running chezmoi apply..."
    chezmoi apply --force >> "$LOG_FILE" 2>&1 \
      && log "chezmoi apply done" \
      || { mark_failed "dotfiles"; warn "chezmoi apply had errors — check $LOG_FILE"; }
  else
    info "Running: chezmoi init --apply $DOTFILES_USER"
    chezmoi init --apply "$DOTFILES_USER" >> "$LOG_FILE" 2>&1 \
      && log "chezmoi init + apply done" \
      || { mark_failed "dotfiles"; warn "chezmoi init/apply had errors — check $LOG_FILE"; }
  fi

  [[ " ${FAILED_GROUPS[*]} " != *" dotfiles "* ]] && mark_installed "dotfiles"
fi

# ---------------------------------------------------------------------------
# Step 9b: Neovim + Mason headless install
# ---------------------------------------------------------------------------
if [[ "$SKIP_NEOVIM" == true ]]; then
  info "neovim — skipped"; mark_skipped "neovim"
else
  # --- neovim binary ---
  if is_installed nvim neovim; then
    info "neovim — already installed, skip"
  else
    pacman_install neovim
  fi

  # --- python-pip: needed by Mason for isort, black, pylint ---
  if pkg_installed python-pip; then
    info "python-pip — already installed, skip"
  else
    pacman_install python-pip
  fi

  # --- python-pynvim: needed by neovim python provider ---
  if python3 -c "import pynvim" &>/dev/null 2>&1; then
    info "pynvim — already installed, skip"
  else
    info "Installing pynvim (neovim python provider)..."
    # CachyOS/Arch: use pacman package to avoid PEP 668 issues
    if pacman -Si python-pynvim &>/dev/null 2>&1; then
      pacman_install python-pynvim
    else
      # Fallback: user-level venv approach if needed
      warn "python-pynvim not in repos — neovim python provider may be missing"
    fi
  fi

  # --- node/npm check for Mason LSPs ---
  if ! has node; then
    # Source nvm if available but node not in PATH
    [[ -s "$NVM_DIR/nvm.sh" ]] && source "$NVM_DIR/nvm.sh" || true
  fi
  if ! has node; then
    warn "node not in PATH — some Mason LSPs (ts_ls, html, css) may fail to install"
  fi

  # --- go check for Mason tools ---
  if ! has go; then
    [[ -s "$HOME/.gvm/scripts/gvm" ]] && source "$HOME/.gvm/scripts/gvm" || true
  fi

  # --- cargo check ---
  if ! has cargo; then
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env" || true
  fi

  # --- headless Lazy.nvim plugin sync ---
  info "Running: nvim --headless '+Lazy! sync' +qa  (this may take a few minutes)..."
  timeout 300 nvim --headless "+Lazy! sync" +qa >> "$LOG_FILE" 2>&1 \
    && log "Lazy.nvim plugin sync complete" \
    || warn "Lazy sync timed out or had errors — non-fatal, plugins may auto-install on first launch"

  # --- headless Mason tool install ---
  # mason-tool-installer has run_on_start=true, so it triggers on VimEnter.
  # We start nvim headlessly and give it up to 3 minutes to finish, then quit.
  info "Running Mason install headlessly (up to 3 minutes)..."
  info "Mason will install: ts_ls html cssls tailwindcss svelte lua_ls graphql"
  info "  emmet_ls prismals pyright gopls marksman markdownlint-cli2"
  info "  prettier stylua isort black pylint gofumpt goimports golangci-lint delve"
  timeout 300 nvim --headless \
    -c "lua vim.defer_fn(function() vim.cmd('qall!') end, 240000)" \
    >> "$LOG_FILE" 2>&1 \
    && log "Mason headless run complete" \
    || warn "Mason headless timed out — non-fatal. Open nvim manually to let Mason finish."

  log "neovim done"; mark_installed "neovim"
fi

step "Step 10/9 — Vault CLI + rclone (Obsidian sync)"

if [[ "$SKIP_VAULT" == true ]]; then
  info "vault-cli — skipped"; mark_skipped "vault-cli"
else
  # --- Rust check (needed to build vault) ---
  if ! has rustc; then
    # Source cargo env if available but rustc not in PATH
    [[ -f "$HOME/.cargo/env" ]] && source "$HOME/.cargo/env" || true
  fi
  if ! has rustc; then
    warn "Rust not installed — vault build will fail. Install Rust first (--skip-rust=false)"
  else
    info "Building vault CLI from source..."
    if [[ -d "$HOME/myfolder/vault-rs" ]]; then
      cd "$HOME/myfolder/vault-rs"
      cargo build --release >> "$LOG_FILE" 2>&1
      mkdir -p "$HOME/.local/bin"
      cp target/release/vault "$HOME/.local/bin/vault"
      log "vault binary built and installed to ~/.local/bin/vault"
    else
      warn "vault-rs source not found at ~/myfolder/vault-rs — skipping build"
    fi
  fi

  # --- rclone config ---
  if has rclone; then
    info "rclone already installed"
  else
    info "Installing rclone..."
    if pacman -Si rclone &>/dev/null 2>&1; then
      pacman_install rclone
    else
      paru_install rclone
    fi
  fi

  # rclone config reconnect (interactive - needs browser)
  if [[ -f "$HOME/.config/rclone/rclone.conf" ]]; then
    info "rclone config found — testing connection..."
    if rclone ls vault: &>> "$LOG_FILE"; then
      log "rclone Google Drive connection OK"
    else
      warn "rclone token expired — run 'rclone config reconnect vault:' manually after bootstrap"
    fi
  else
    warn "No rclone config found — run 'rclone config create vault: drive' manually"
  fi

  # --- Sync test ---
  if has vault && has rclone; then
    info "Running initial vault export + sync test..."
    vault export >> "$LOG_FILE" 2>&1
    rclone sync ~/myfolder/obsidian-vault vault:myfolder/obsidian-vault >> "$LOG_FILE" 2>&1 \
      && log "Initial vault export + sync OK" \
      || warn "Initial sync had issues — check $LOG_FILE"
  fi

  log "vault-cli done"; mark_installed "vault-cli"
fi

if [[ "$SKIP_TMUX" == true ]]; then
  info "tmux — skipped"; mark_skipped "tmux"
else
  # Install tmux
  if is_installed tmux; then
    info "tmux — already installed, skip"
  else
    pacman_install tmux
  fi

  # Install TPM (Tmux Plugin Manager)
  if [[ -d "$HOME/.tmux/plugins/tpm" ]]; then
    info "TPM — already installed, skip"
  else
    info "Installing TPM (Tmux Plugin Manager)..."
    mkdir -p "$HOME/.tmux/plugins"
    git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm >> "$LOG_FILE" 2>&1 \
      && log "TPM installed to ~/.tmux/plugins/tpm" \
      || { warn "TPM installation failed"; warn "Manual install: git clone https://github.com/tmux-plugins/tpm ~/.tmux/plugins/tpm"; }
  fi
  
  # Fix permissions (WSL sometimes has issues)
  if [[ -d "$HOME/.tmux/plugins/tpm" ]]; then
    chmod -R u+rwx "$HOME/.tmux/plugins/tpm" 2>/dev/null || true
  fi

  info "To activate TPM plugins:"
  info "  1. Start tmux: tmux"
  info "  2. Press: Ctrl+a, I (capital I)"
  info "  3. Wait 2-3 seconds for plugins to download"
  info "Plugins: vim-tmux-navigator, tmux-resurrect, tmux-continuum"

  log "tmux done"; mark_installed "tmux"
fi

# ---------------------------------------------------------------------------
# Default shell prompt
# ---------------------------------------------------------------------------
FISH_PATH=$(which fish 2>/dev/null || true)
ZSH_PATH=$(which zsh 2>/dev/null || true)
CURRENT_SHELL=$(getent passwd "$USER" | cut -d: -f7)

if [[ "$SKIP_FISH" == false || "$SKIP_ZSH" == false ]]; then
  if [[ "$NON_INTERACTIVE" == false ]]; then
    step "Default shell"
    echo "  Current default shell: $CURRENT_SHELL"
    echo ""
    OPTIONS=("keep current ($CURRENT_SHELL)")
    [[ -n "$FISH_PATH" ]] && OPTIONS+=("fish ($FISH_PATH)")
    [[ -n "$ZSH_PATH"  ]] && OPTIONS+=("zsh ($ZSH_PATH)")

    echo "  Choose default shell:"
    for i in "${!OPTIONS[@]}"; do
      echo "    $((i+1))) ${OPTIONS[$i]}"
    done
    echo ""
    read -rp "  Choice [1]: " SHELL_CHOICE
    SHELL_CHOICE="${SHELL_CHOICE:-1}"

    if [[ "$SHELL_CHOICE" -ge 2 ]]; then
      IDX=$((SHELL_CHOICE - 1))
      NEW_SHELL_LINE="${OPTIONS[$IDX]}"
      # Extract path from "name (/path)"
      NEW_SHELL_PATH=$(echo "$NEW_SHELL_LINE" | grep -oP '\(.*\)' | tr -d '()')
      if [[ -n "$NEW_SHELL_PATH" ]]; then
        # Add to /etc/shells if not there
        grep -qxF "$NEW_SHELL_PATH" /etc/shells || echo "$NEW_SHELL_PATH" | sudo tee -a /etc/shells | tee -a "$LOG_FILE" >/dev/null
        chsh -s "$NEW_SHELL_PATH"
        log "Default shell changed to $NEW_SHELL_PATH (takes effect on next login)"
      fi
    fi
  fi
fi

# ---------------------------------------------------------------------------
# Final summary
# ---------------------------------------------------------------------------
step "Done"
echo ""
if [[ ${#INSTALLED_GROUPS[@]} -gt 0 ]]; then
  echo -e "${GREEN}Installed:${NC}"
  for g in "${INSTALLED_GROUPS[@]}"; do echo "  + $g"; done
fi
if [[ ${#SKIPPED_GROUPS[@]} -gt 0 ]]; then
  echo -e "${DIM}Skipped:${NC}"
  for g in "${SKIPPED_GROUPS[@]}"; do echo "    $g"; done
fi
if [[ ${#FAILED_GROUPS[@]} -gt 0 ]]; then
  echo -e "${RED}Failed (check log):${NC}"
  for g in "${FAILED_GROUPS[@]}"; do echo "  ! $g"; done
fi
echo ""
echo "  Full log: $LOG_FILE"
echo ""

# Reminders
if [[ "$SKIP_DOTFILES" == false ]] && [[ ! -f "$HOME/.config/chezmoi/key.txt" ]]; then
  warn "Reminder: age key not set up — encrypted dotfiles were not decrypted."
  warn "  Place your key at ~/.config/chezmoi/key.txt then run: chezmoi apply"
fi
if [[ "$SKIP_NEOVIM" == false ]]; then
  info "Reminder: open nvim once to let Mason finish any remaining installs."
fi
if [[ "$SKIP_ZSH" == false ]]; then
  info "Reminder: zsh plugins will clone themselves on first interactive zsh session."
fi
echo ""
