//! Capture cepat: daily note, halaman wiki, catatan bebas.

use crate::util::{append_to_file, sanitize_filename, vault_root, write_file};
use anyhow::Result;
use chrono::Local;

/// Daily note: 00_Daily/YYYY-MM-DD.md. Buat dari template daily-standup.md bila belum ada,
/// lalu append entri bertimestamp bila ada teks.
pub fn daily(text: &str) -> Result<()> {
    let now = Local::now();
    let date = now.format("%Y-%m-%d").to_string();
    let time = now.format("%H:%M").to_string();
    let file = vault_root().join("00_Daily").join(format!("{date}.md"));

    if !file.exists() {
        // CLI pakai body inline terisi penuh (tanggal langsung diisi), BUKAN template
        // Obsidian yang pakai sintaks core {{date}}. Dengan begitu daily dari CLI dan
        // dari Calendar/Daily Notes sama-sama terisi benar (lihat PKM: Opsi X).
        use chrono::Datelike;
        let prev = (now - chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        let next = (now + chrono::Duration::days(1)).format("%Y-%m-%d").to_string();
        let hari = match now.weekday() {
            chrono::Weekday::Mon => "Senin",
            chrono::Weekday::Tue => "Selasa",
            chrono::Weekday::Wed => "Rabu",
            chrono::Weekday::Thu => "Kamis",
            chrono::Weekday::Fri => "Jumat",
            chrono::Weekday::Sat => "Sabtu",
            chrono::Weekday::Sun => "Minggu",
        };
        let bulan = [
            "Januari", "Februari", "Maret", "April", "Mei", "Juni", "Juli", "Agustus",
            "September", "Oktober", "November", "Desember",
        ][(now.month() - 1) as usize];
        let judul = format!("{hari}, {:02} {bulan} {}", now.day(), now.year());
        let body = format!(
            "---\ndate: \"{date}\"\ntype: daily\ntags:\n  - daily\ntasks_done: 0\n---\n# 📅 {judul}\n\n> [!tip]- Navigasi\n> ⬅️ [[{prev}|Kemarin]] · [[{next}|Besok]] ➡️ · 🗂️ [[Board|Board nego-frontend]] · 📊 [[Dashboard]]\n\n## 🎯 Task Hari Ini\n> Tarik dari Board. Boleh link ke file task/project.\n\n- [ ] \n- [ ] \n\n## 📋 Task Project (belum selesai)\n```dataview\nTABLE WITHOUT ID\n  file.link AS \"Task\",\n  project AS \"Project\",\n  status AS \"Status\"\nFROM \"10_Projects\"\nWHERE type = \"task\" AND status != \"done\"\nSORT status ASC, file.name ASC\n```\n\n## 🔄 Progress Subtask\n```dataviewjs\nconst files = dv.pages('\"10_Projects\"').where(p => p.type === \"task\" && p.status !== \"done\");\nif (!files.length) {{ dv.paragraph(\"_Tidak ada task aktif._\"); }}\nfor (const f of files) {{\n  const content = await dv.io.load(f.file.path);\n  const sec = content.match(/##\\s*Subtask([\\s\\S]*?)(?:\\n##\\s|\\n*$)/i);\n  if (!sec) continue;\n  const subs = [...sec[1].matchAll(/^>?\\s*-\\s*\\[( |x)\\]\\s*(.+)$/gm)].filter(s => s[2].trim());\n  if (!subs.length) continue;\n  const done = subs.filter(s => s[1] === \"x\").length;\n  dv.header(4, `${{f.file.link}} — ${{done}}/${{subs.length}}`);\n  dv.list(subs.map(s => (s[1] === \"x\" ? \"✅ \" : \"⬜ \") + s[2].replace(/\\*\\*/g, \"\")));\n}}\n```\n\n## 🕒 Work Log\n- \n\n## 💡 Catatan & Insight\n- \n\n## 🔄 Refleksi\n- **Selesai hari ini:** \n- **Belum selesai → besok:** \n- **Blocker:** \n"
        );
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
    let date = Local::now().format("%Y-%m-%d").to_string();
    let slug = crate::util::slugify(title);
    let file = vault_root()
        .join("30_Resources/LLM_Wiki")
        .join(format!("{date}-{slug}.md"));

    if !file.exists() {
        // Template `wiki.md` di vault memakai JS Templater (khusus Obsidian) — CLI TIDAK memuatnya.
        // CLI pakai body bawaan di bawah (properties detail, tanpa JS).
        let body = format!(
            "---\ntitle: \"{safe}\"\ntype: wiki\ncreated: \"{date}\"\nupdated: \"{date}\"\naliases: []\ntags:\n  - wiki\n  - concept\n---\n\n> [!note] {safe}\n> Ringkasan 1–2 kalimat: apa ini & kenapa penting. · Dibuat: {date}\n\n# {safe}\n\n## 🧠 Inti Konsep\n- \n\n## 📑 Detail\n- \n\n## 🔗 Kaitan\n- \n\n## 📎 Sumber\n- \n"
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

/// Catatan bebas project: 10_Projects/<nama>/notes/<judul>.md
/// Dijalankan dari dalam folder project kode (deteksi via .git).
pub fn pnote(title: &str, text: &str) -> Result<()> {
    let p = crate::project::require()?;
    let display = sanitize_filename(title);
    if display.is_empty() {
        anyhow::bail!("judul catatan kosong");
    }
    let slug = crate::util::slugify(title);
    let date_slug = Local::now().format("%Y-%m-%d").to_string();
    let file = p.vault_dir().join("notes").join(format!("{date_slug}-{slug}.md"));
    let date_full = Local::now().format("%Y-%m-%d %H:%M").to_string();
    let date = Local::now().format("%Y-%m-%d").to_string();

    if !file.exists() {
        // Catatan: template `project-note.md` di vault memakai JS Templater (khusus Obsidian),
        // jadi CLI TIDAK memuatnya (akan bocor mentah). CLI pakai body bawaan di bawah —
        // properties detail + tag project, konsisten dengan hasil Obsidian.
        let body = format!(
            "---\ntitle: \"{display}\"\ntype: project-note\nproject: \"{proj}\"\ncreated: \"{date}\"\nupdated: \"{date}\"\naliases: []\ntags:\n  - note\n  - {proj}\n---\n\n> [!info] {display}\n> Project: **{proj}** · Dibuat: {date}\n\n# {display}\n\n## 📌 Ringkasan\n- \n\n## 📝 Catatan\n- \n\n## 🔗 Terkait\n- \n",
            display = display,
            proj = p.name,
            date = date
        );
        write_file(&file, &body)?;
        println!("✓ dibuat catatan project {}", file.display());
    }
    if !text.trim().is_empty() {
        append_to_file(&file, &format!("- **{date_full}**: {text}\n"))?;
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
