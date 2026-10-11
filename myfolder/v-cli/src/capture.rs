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
        let mut vars = HashMap::new();
        vars.insert("TITLE", safe.clone());
        vars.insert("DATE", date.clone());
        vars.insert("TAG", "concept".to_string());
        let body = crate::util::load_template("wiki", &vars).unwrap_or_else(|| {
            format!("---\ntitle: \"{safe}\"\ntype: wiki\nupdated: {date}\n---\n\n# {safe}\n")
        });
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

/// Catatan bebas project: 10_Projects/<nama>/notes/<judul>.md
/// Dijalankan dari dalam folder project kode (deteksi via .git).
pub fn pnote(title: &str, text: &str) -> Result<()> {
    let p = crate::project::require()?;
    let display = sanitize_filename(title);
    if display.is_empty() {
        anyhow::bail!("judul catatan kosong");
    }
    let slug = crate::util::slugify(title);
    let file = p.vault_dir().join("notes").join(format!("{slug}.md"));
    let date = Local::now().format("%Y-%m-%d %H:%M").to_string();

    if !file.exists() {
        let mut vars = HashMap::new();
        vars.insert("TITLE", display.clone());
        vars.insert("PROJECT", p.name.clone());
        vars.insert("DATE", date.clone());
        vars.insert("CONTENT", String::new());
        let body = crate::util::load_template("project-note", &vars).unwrap_or_else(|| {
            format!("---\ntitle: \"{display}\"\ntype: project-note\nproject: {}\ncreated: {date}\n---\n\n# {display}\n\n", p.name)
        });
        write_file(&file, &body)?;
        println!("✓ dibuat catatan project {}", file.display());
    }
    if !text.trim().is_empty() {
        append_to_file(&file, &format!("- **{date}**: {text}\n"))?;
        println!("✓ isi ditambahkan ke '{display}' (project {})", p.name);
    }
    Ok(())
}

/// Daftar catatan bebas untuk project saat ini.
pub fn pnotes_list() -> Result<()> {
    let p = crate::project::require()?;
    let dir = p.vault_dir().join("notes");
    if !dir.exists() {
        println!("(belum ada catatan untuk project {}. Buat dengan: v pnote \"judul\" \"isi\")", p.name);
        return Ok(());
    }
    let mut found = false;
    println!("Catatan project {}:", p.name);
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut entries: Vec<_> = rd.flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
            .collect();
        entries.sort();
        for path in entries {
            if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                println!("  - {name}");
                found = true;
            }
        }
    }
    if !found {
        println!("  (kosong)");
    }
    Ok(())
}
