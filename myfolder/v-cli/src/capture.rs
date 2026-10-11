//! Capture cepat: daily note, halaman wiki, catatan bebas.

use crate::util::{
    append_to_file, render_template, sanitize_filename, templates_dir, vault_root, write_file,
};
use anyhow::Result;
use chrono::Local;
use std::collections::HashMap;

/// Daily note: 00_Daily/YYYY-MM-DD.md. Buat dari template daily-standup.md bila belum ada,
/// lalu append entri bertimestamp bila ada teks.
pub fn daily(text: &str) -> Result<()> {
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let time = now.format("%H:%M").to_string();
    let file = vault_root().join("00_Daily").join(format!("{date}.md"));

    if !file.exists() {
        let tpl_path = templates_dir().join("daily-standup.md");
        let body = if tpl_path.exists() {
            let tpl = std::fs::read_to_string(&tpl_path)?;
            let mut vars = HashMap::new();
            vars.insert("DATE", date.clone());
            vars.insert("TIME", time.clone());
            render_template(&tpl, &vars)
        } else {
            format!("# {date}\n\n## Work Log\n")
        };
        write_file(&file, &body)?;
        println!("✓ dibuat daily note {}", file.display());
    }

    if !text.trim().is_empty() {
        let entry = format!("- **{time}**: {text}\n");
        append_to_file(&file, &entry)?;
        println!("✓ entri ditambahkan ke daily {date}");
    }
    Ok(())
}

/// Halaman wiki: 30_Resources/LLM_Wiki/<Judul>.md. Buat dengan frontmatter schema bila baru,
/// append isi bila ada.
pub fn wiki(title: &str, text: &str) -> Result<()> {
    let safe = sanitize_filename(title);
    let file = vault_root()
        .join("30_Resources/LLM_Wiki")
        .join(format!("{safe}.md"));
    let date = Local::now().format("%Y-%m-%d").to_string();

    if !file.exists() {
        let body = format!(
            "---\ntitle: \"{safe}\"\ntype: wiki\ntags: [llm]\nupdated: {date}\n---\n\n\
             # {safe}\n\n> Ringkasan singkat.\n\n## Inti Konsep\n\n## Kaitan\n\n## Sumber\n"
        );
        write_file(&file, &body)?;
        println!("✓ dibuat halaman wiki {}", file.display());
    }
    if !text.trim().is_empty() {
        append_to_file(&file, &format!("\n{text}\n"))?;
        println!("✓ isi ditambahkan ke wiki '{safe}'");
    }
    Ok(())
}

/// Catatan bebas ke path relatif terhadap vault.
pub fn note(rel_path: &str, text: &str) -> Result<()> {
    // Cegah path traversal keluar vault
    if rel_path.contains("..") {
        anyhow::bail!("path tidak boleh mengandung '..'");
    }
    let file = vault_root().join(rel_path);
    if !text.trim().is_empty() {
        append_to_file(&file, &format!("{text}\n"))?;
        println!("✓ ditulis ke {}", file.display());
    } else if !file.exists() {
        write_file(&file, "")?;
        println!("✓ dibuat {}", file.display());
    } else {
        println!("(tanpa isi; file sudah ada: {})", file.display());
    }
    Ok(())
}
