//! Utilitas bersama: resolusi path vault, substitusi template, helper file.

use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::PathBuf;

/// Root vault PARA. Override dengan env `V_VAULT`.
pub fn vault_root() -> PathBuf {
    if let Ok(p) = std::env::var("V_VAULT") {
        return PathBuf::from(p);
    }
    home().join("myfolder/obsidian-vault")
}

/// Folder template. Override dengan env `V_TEMPLATES`.
pub fn templates_dir() -> PathBuf {
    if let Ok(p) = std::env::var("V_TEMPLATES") {
        return PathBuf::from(p);
    }
    vault_root().join("30_Resources/Templates")
}

pub fn home() -> PathBuf {
    std::env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."))
}

/// Nama file aman untuk Windows/NTFS: buang karakter ilegal `" * : < > ? \ / |`.
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '"' | '*' | ':' | '<' | '>' | '?' | '\\' | '/' | '|' => '-',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Slug untuk nama folder: huruf kecil, spasi -> '-'.
pub fn slugify(name: &str) -> String {
    sanitize_filename(name).to_lowercase().replace(' ', "-")
}

/// Substitusi placeholder gaya `{{ VAR }}` dan Templater `<% ... %>` yang umum.
///
/// - `{{ KEY }}` diganti dari `vars` (spasi di dalam kurung diabaikan).
/// - `<% tp.date.now(...) %>` dan `<% tp.file.title %>` diisi bila ada di `vars`
///   sebagai TP_DATE / TP_TITLE; sintaks `<% %>` lain dibiarkan utuh agar
///   Templater di Obsidian memprosesnya.
pub fn render_template(tpl: &str, vars: &HashMap<&str, String>) -> String {
    let mut out = String::with_capacity(tpl.len());
    let mut rest = tpl;
    while !rest.is_empty() {
        if let Some(pos) = rest.find("{{").or_else(|| rest.find("<%")) {
            // emit teks sebelum penanda
            out.push_str(&rest[..pos]);
            rest = &rest[pos..];
            if let Some(stripped) = rest.strip_prefix("{{") {
                if let Some(end) = stripped.find("}}") {
                    let key = stripped[..end].trim();
                    if let Some(val) = vars.get(key) {
                        out.push_str(val);
                    } else {
                        out.push_str(&rest[..end + 4]); // pertahankan {{ ... }}
                    }
                    rest = &stripped[end + 2..];
                    continue;
                }
            } else if let Some(stripped) = rest.strip_prefix("<%") {
                if let Some(end) = stripped.find("%>") {
                    let expr = stripped[..end].trim();
                    match render_templater_expr(expr, vars) {
                        Some(val) => out.push_str(&val),
                        None => out.push_str(&rest[..end + 4]), // pertahankan <% ... %>
                    }
                    rest = &stripped[end + 2..];
                    continue;
                }
            }
            // penanda tak lengkap: emit 2 char lalu lanjut
            let mut ch = rest.chars();
            if let Some(c) = ch.next() {
                out.push(c);
            }
            rest = ch.as_str();
        } else {
            out.push_str(rest);
            break;
        }
    }
    out
}

/// Isi ekspresi Templater umum. Kembalikan None agar sintaks dibiarkan utuh.
fn render_templater_expr(expr: &str, vars: &HashMap<&str, String>) -> Option<String> {
    let e = expr.replace(' ', "");
    if e.starts_with("tp.date.now") {
        return vars.get("DATE").cloned();
    }
    if e.starts_with("tp.file.title") {
        return vars.get("TITLE").cloned();
    }
    None
}

/// Muat template bernama `<name>.md` dari folder Templates dan render dengan `vars`.
/// Bila template tidak ada, kembalikan None (pemanggil sediakan fallback).
pub fn load_template(name: &str, vars: &HashMap<&str, String>) -> Option<String> {
    let path = templates_dir().join(format!("{name}.md"));
    let tpl = std::fs::read_to_string(&path).ok()?;
    Some(render_template(&tpl, vars))
}

/// Append teks ke file, buat file+parent bila belum ada.
pub fn append_to_file(path: &std::path::Path, content: &str) -> Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("buat folder {}", parent.display()))?;
    }
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("buka {}", path.display()))?;
    f.write_all(content.as_bytes())
        .with_context(|| format!("tulis {}", path.display()))?;
    Ok(())
}

/// Tulis file (timpa), buat parent bila perlu.
pub fn write_file(path: &std::path::Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("buat folder {}", parent.display()))?;
    }
    std::fs::write(path, content).with_context(|| format!("tulis {}", path.display()))?;
    Ok(())
}

pub fn print_config() -> Result<()> {
    println!("v — konfigurasi aktif:");
    println!("  Vault root : {}", vault_root().display());
    println!("  Templates  : {}", templates_dir().display());
    println!("  CWD        : {}", std::env::current_dir()?.display());
    match crate::project::detect() {
        Some(p) => {
            println!("  Project    : {} ({})", p.name, p.code_dir.display());
            println!("  Context MD : {}", p.vault_context_file().display());
            println!("  AGENTS.md  : {}", p.agents_file().display());
        }
        None => println!("  Project    : (tidak terdeteksi — tidak di dalam folder project git)"),
    }
    Ok(())
}

/// Panduan penggunaan & workflow lengkap.
pub fn print_guide() {
    let g = r#"
╔══════════════════════════════════════════════════════════════╗
║  v — File-first PARA vault CLI                                 ║
╚══════════════════════════════════════════════════════════════╝

KONSEP
  • Semua catatan = file markdown di vault PARA (~/myfolder/obsidian-vault).
  • Tidak ada database. Obsidian & v menulis file yang sama.
  • Konteks AI per-project hidup di folder KODE (AGENTS.md), di-mirror ke vault.
  • Sinkron antar-device (Windows/HP/CachyOS) lewat git: `vsync`.

────────────────────────────────────────────────────────────────
KONTEKS AI PER-PROJECT  (jalankan dari dalam folder kode project)
────────────────────────────────────────────────────────────────
  v ctx                 Tampilkan konteks project ini
  v ctx init            Buat AGENTS.md awal (otomatis bila belum ada)
  v ctx log "..."       Tambah entri log (keputusan, bug, progress)
  v ctx show            Cetak konteks (untuk di-feed ke AI)
  v ctx link            Tautkan AGENTS.md -> CLAUDE.md/GEMINI.md/.kiro/.hermes
                        (symlink, aman, tak menimpa file yang ada)
  ↳ Tulis ke: <project>/AGENTS.md  +  mirror 10_Projects/<nama>/_context.md

────────────────────────────────────────────────────────────────
TASK PER-PROJECT  (dari dalam folder kode project)
────────────────────────────────────────────────────────────────
  v task "judul"        Tambah task (1 task = 1 file dari template task.md)
  v task done N         Tandai task nomor N selesai (ubah status frontmatter)
  v tasks --here        Task aktif project ini  (alias: vt --here)
  v tasks               Task aktif semua project
  ↳ Tulis ke: 10_Projects/<nama>/tasks/<judul>.md

────────────────────────────────────────────────────────────────
CATATAN BEBAS PROJECT  (dari dalam folder kode project)
────────────────────────────────────────────────────────────────
  v pnote "judul" "isi" Catatan bebas (template project-note.md)
  v pnotes              Daftar catatan project ini
  ↳ Tulis ke: 10_Projects/<nama>/notes/<judul>.md
  ↳ Bisa juga ditulis langsung di Obsidian di folder yang sama.

────────────────────────────────────────────────────────────────
CAPTURE CEPAT  (dari mana saja)
────────────────────────────────────────────────────────────────
  v daily "..."         Entri daily  (template daily-standup.md) -> 00_Daily/
  v wiki "Judul" "..."  Halaman wiki (template wiki.md)          -> 30_Resources/LLM_Wiki/
  v note <path> "..."   Catatan bebas ke path PARA mana pun

────────────────────────────────────────────────────────────────
MENAMPILKAN & MENCARI
────────────────────────────────────────────────────────────────
  v list daily          Daftar daily notes
  v list wiki           Daftar halaman wiki
  v list notes          Daftar catatan bebas project
  v list projects       Daftar project
  v search "kata"       Cari di seluruh vault (ripgrep)
  v stats               Statistik vault

────────────────────────────────────────────────────────────────
SINKRON & INFO
────────────────────────────────────────────────────────────────
  vsync ["pesan"]       pull --rebase -> commit -> push vault ke git
  v config              Path & project yang terdeteksi
  v guide               Panduan ini

CATATAN: semua catatan dibuat dari template di 30_Resources/Templates/.
         Ubah template di sana → output v ikut berubah.

────────────────────────────────────────────────────────────────
ALIAS PENDEK (zsh)
────────────────────────────────────────────────────────────────
  vc=v ctx   vt=v tasks   vs=v search   vd=v daily   vw=v wiki   vst=v stats

════════════════════════════════════════════════════════════════
CONTOH WORKFLOW: vibe-coding project "avinet" dengan AI
════════════════════════════════════════════════════════════════
  cd ~/myfolder/work/avinet        # masuk folder kode project
  v ctx link                       # (sekali) tautkan ke semua AI tool
  v ctx log "mulai: refactor auth" # catat konteks
  # ... ngoding dengan kiro/opencode/claude (mereka baca AGENTS.md) ...
  v ctx log "fixed: token refresh race, pakai mutex"
  v task "tulis test auth"         # catat task
  v task done 1                    # selesaikan task
  v pnote "meeting" "deadline Q4"  # catatan bebas
  vsync "update avinet context"    # sinkron ke Obsidian Windows/HP/CachyOS

Jalankan `v <perintah> --help` untuk detail tiap perintah.
"#;
    println!("{g}");
}
