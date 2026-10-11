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
