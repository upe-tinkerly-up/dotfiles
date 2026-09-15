# dotfiles

Managed with [chezmoi](https://chezmoi.io).

## Install di mesin baru

Install chezmoi lalu apply sekaligus:

    sh -c "$(curl -fsLS get.chezmoi.io)" -- init --apply upe-tinkerly-up

Atau kalau chezmoi sudah ada:

    chezmoi init --apply upe-tinkerly-up

## Secrets

Secrets di-enkripsi dengan age. Pastikan key sudah ada di
`~/.config/chezmoi/key.txt` sebelum menjalankan `chezmoi apply`.

Restore key dari backup, lalu:

    chezmoi apply

## Isi

- Shell: bash, zsh (dengan plugins: fzf-tab, autosuggestions, fast-syntax-highlighting, vi-mode, history-substring-search)
- Editor: neovim (LazyVim-based)
- Terminal: alacritty, kitty
- WM: hyprland
- Prompt: starship
- File manager: lf
- Multiplexer config: -
- Tools: btop, cava, fastfetch, fish
