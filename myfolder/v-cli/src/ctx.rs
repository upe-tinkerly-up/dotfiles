//! Konteks AI per-project.
//!
//! Sumber kebenaran: `<folder-project>/AGENTS.md` (dibaca opencode, dan via
//! symlink bisa jadi CLAUDE.md/GEMINI.md). Di-mirror ke vault
//! `10_Projects/<nama>/_context.md` agar terbaca di Obsidian & tersinkron git.

use crate::project::{self, Project};
use crate::util::{append_to_file, load_template, write_file};
use anyhow::{Context, Result};
use chrono::Local;
use std::collections::HashMap;

/// Konteks AI dari template `ai-context.md` (fallback ke kerangka internal bila template hilang).
fn context_body(name: &str, date: &str) -> String {
    let mut vars = HashMap::new();
    vars.insert("PROJECT", name.to_string());
    vars.insert("DATE", date.to_string());
    vars.insert("TITLE", name.to_string());
    load_template("ai-context", &vars).unwrap_or_else(|| {
        format!(
            "# AGENTS.md — {name}\n\n> Konteks AI project {name}.\n\n\
             ## Log Aktivitas\n- **{date}**: konteks diinisialisasi.\n"
        )
    })
}

/// Inisialisasi konteks untuk project saat ini bila belum ada.
/// Kembalikan Project agar bisa dipakai pemanggil.
pub fn init() -> Result<Project> {
    let p = project::require()?;
    let date = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let agents = p.agents_file();

    if !agents.exists() {
        let body = context_body(&p.name, &date);
        write_file(&agents, &body)?;
        println!("✓ dibuat {}", agents.display());
    }
    // Mirror ke vault
    mirror_to_vault(&p)?;
    Ok(p)
}

/// Salin AGENTS.md -> vault _context.md (mirror satu arah: kode -> vault).
fn mirror_to_vault(p: &Project) -> Result<()> {
    let agents = p.agents_file();
    if agents.exists() {
        let content = std::fs::read_to_string(&agents)?;
        write_file(&p.vault_context_file(), &content)?;
    }
    Ok(())
}

/// Tambah entri log bertimestamp ke bagian "Log Aktivitas".
pub fn log(text: &str) -> Result<()> {
    if text.trim().is_empty() {
        anyhow::bail!("isi log kosong. Pakai: v ctx log \"pesan\"");
    }
    let p = project::require()?;
    let agents = p.agents_file();
    if !agents.exists() {
        init()?;
    }
    let ts = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let entry = format!("- **{ts}**: {text}\n");
    append_to_file(&agents, &entry)?;
    mirror_to_vault(&p)?;
    println!("✓ log ditambahkan ke {} (+ mirror vault)", p.name);
    Ok(())
}

/// Cetak konteks project ke stdout (untuk di-feed ke AI).
pub fn show() -> Result<()> {
    let p = project::require()?;
    let agents = p.agents_file();
    if !agents.exists() {
        println!("(belum ada konteks untuk '{}'. Jalankan `v ctx init`.)", p.name);
        return Ok(());
    }
    let content = std::fs::read_to_string(&agents)?;
    print!("{content}");
    Ok(())
}

/// Tautkan AGENTS.md ke pintu masuk tool lain via symlink (opt-in, aman).
///
/// Hanya di folder kode project (Linux-only). TIDAK pernah menyentuh vault,
/// jadi git Windows pada vault tetap aman. Tidak menimpa file yang sudah ada.
pub fn link() -> Result<()> {
    let p = project::require()?;
    let agents = p.agents_file();
    if !agents.exists() {
        init()?;
    }

    // Daftar target: (path relatif thd code_dir, target symlink relatif thd lokasi link)
    // Target selalu menunjuk ke AGENTS.md di root project.
    let single_file = [("CLAUDE.md", "AGENTS.md"), ("GEMINI.md", "AGENTS.md")];
    let nested = [
        (".kiro/steering/00-agents.md", "../../AGENTS.md"),
        (".hermes/AGENTS.md", "../AGENTS.md"),
    ];

    let mut created = 0;
    let mut skipped = 0;

    for (link_rel, target) in single_file {
        let link_path = p.code_dir.join(link_rel);
        created += make_symlink(&link_path, std::path::Path::new(target), link_rel, &mut skipped)?;
    }
    for (link_rel, target) in nested {
        let link_path = p.code_dir.join(link_rel);
        if let Some(parent) = link_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        created += make_symlink(&link_path, std::path::Path::new(target), link_rel, &mut skipped)?;
    }

    println!(
        "✓ ctx link project {}: {created} symlink dibuat, {skipped} dilewati (sudah ada).",
        p.name
    );
    println!("  (symlink hanya di folder kode Linux; vault & git Windows tidak tersentuh)");
    Ok(())
}

/// Buat symlink bila belum ada. Kembalikan 1 bila dibuat, 0 bila dilewati.
/// Tidak menimpa file/symlink yang sudah ada (keamanan terhadap konteks nyata).
fn make_symlink(
    link_path: &std::path::Path,
    target: &std::path::Path,
    label: &str,
    skipped: &mut i32,
) -> Result<i32> {
    use std::os::unix::fs::symlink;
    // symlink_metadata: deteksi symlink/ file tanpa mengikuti target
    if link_path.symlink_metadata().is_ok() {
        println!("  - {label}: sudah ada, dilewati");
        *skipped += 1;
        return Ok(0);
    }
    symlink(target, link_path)
        .with_context(|| format!("buat symlink {label}"))?;
    println!("  - {label} -> {}", target.display());
    Ok(1)
}
