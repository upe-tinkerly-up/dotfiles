# Better ls
alias ls='eza --icons'

# Detailed listing
alias ll='eza -lh --icons --git'

# Detailed listing including hidden files
alias la='eza -lah --icons --git'

# Tree view
alias tree='eza --tree --icons'

# Reuse ls completions for eza (avoids defining a separate completion function)
compdef eza=ls

# Better cat
alias cat='bat'

# =========================================================
# Core utilities
# =========================================================

alias grep='rg --color=auto'
alias diff='diff --color=auto'
alias df='df -h'

# =========================================================
# Navigation
# =========================================================

alias -- -='cd -'  # -- prevents - being parsed as a flag; cd - jumps to previous directory

lf() { # zsh follow lf navigation
    tmp=$(mktemp)
    command lf -last-dir-path="$tmp" "$@"
    if [ -f "$tmp" ]; then
        dir=$(cat "$tmp")
        rm -f "$tmp"
        [ -d "$dir" ] && [ "$dir" != "$(pwd)" ] && cd "$dir"
    fi
}

# =========================================================
# Editor
# =========================================================

alias vim='nvim'

# =========================================================
# Git
# =========================================================

alias glog='PAGER="less -F -X" git log'                              # -F quit if one screen, -X no clear on exit
alias gadog='PAGER="less -F -X" git log --all --decorate --oneline --graph'
alias dotfiles='git --git-dir=$HOME/.dotfiles --work-tree=$HOME'

# =========================================================
# Video
# =========================================================

alias stream='mpv av://v4l2:/dev/video4 --fullscreen --demuxer-lavf-o=input_format=mjpeg,framerate=30 --profile=low-latency --untimed'

# =========================================================
# v — File-first PARA vault CLI
# =========================================================
# Vault PARA root (dipakai `v` dan fungsi vsync).
export V_VAULT="$HOME/myfolder/obsidian-vault"
export VAULT="$V_VAULT"   # kompatibilitas lama untuk vsync

# Shortcut capture & query
alias vt="v tasks"
alias vs="v search"
alias vst="v stats"
alias vd="v daily"
alias vw="v wiki"
alias vc="v ctx"

# vsync: sinkronisasi vault PARA lewat git (pull --rebase, commit, push).
vsync() {
  local vault="${V_VAULT:-$HOME/myfolder/obsidian-vault}"
  local msg="${1:-vault sync $(date '+%Y-%m-%d %H:%M')}"
  git -C "$vault" pull --rebase --autostash || { echo "vsync: pull gagal, selesaikan dulu"; return 1; }
  if [ -n "$(git -C "$vault" status --porcelain)" ]; then
    git -C "$vault" add -A
    git -C "$vault" commit -m "$msg"
  else
    echo "vsync: tidak ada perubahan lokal"
  fi
  git -C "$vault" push
}
