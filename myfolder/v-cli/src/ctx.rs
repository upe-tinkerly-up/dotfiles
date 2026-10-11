//! Konteks AI per-project.
//!
//! Sumber kebenaran: `<folder-project>/AGENTS.md` (dibaca opencode, dan via
//! symlink bisa jadi CLAUDE.md/GEMINI.md). Di-mirror ke vault
//! `10_Projects/<nama>/_context.md` agar terbaca di Obsidian & tersinkron git.

use crate::project::{self, Project};
use crate::util::{append_to_file, write_file};
use anyhow::Result;
use chrono::Local;

/// Template konteks AI — kerangka fleksibel, universal lintas jenis project.
fn context_template(name: &str, date: &str) -> String {
    format!(
        "# AGENTS.md — {name}\n\
         \n\
         > Konteks persisten untuk asisten AI pada project **{name}**.\n\
         > Diperbarui lewat `v ctx log \"...\"`. Dibaca oleh opencode/kiro/claude dsb.\n\
         \n\
         ## Ringkasan\n\
         - Apa project ini, tujuan, stack utama.\n\
         \n\
         ## Arsitektur\n\
         - Struktur & komponen penting.\n\
         \n\
         ## Keputusan\n\
         - Keputusan desain penting beserta alasannya.\n\
         \n\
         ## Bug & Solusi\n\
         - Catatan bug yang pernah diperbaiki + akar masalah.\n\
         \n\
         ## State Terkini\n\
         - Sedang dikerjakan apa sekarang.\n\
         \n\
         ## Task Aktif\n\
         - Lihat `tasks.md` (atau `v tasks --here`).\n\
         \n\
         ## Log Aktivitas\n\
         _Entri otomatis dari `v ctx log`._\n\
         \n\
         - **{date}**: konteks diinisialisasi.\n"
    )
}

/// Inisialisasi konteks untuk project saat ini bila belum ada.
/// Kembalikan Project agar bisa dipakai pemanggil.
pub fn init() -> Result<Project> {
    let p = project::require()?;
    let date = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let agents = p.agents_file();

    if !agents.exists() {
        let body = context_template(&p.name, &date);
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
