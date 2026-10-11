//! Task per-project, disimpan sebagai checkbox markdown di
//! vault 10_Projects/<nama>/tasks.md (kompatibel Obsidian & Dataview).

use crate::project;
use crate::util::{append_to_file, vault_root, write_file};
use anyhow::Result;
use chrono::Local;

fn ensure_header(path: &std::path::Path, project_name: &str) -> Result<()> {
    if !path.exists() {
        let header = format!(
            "---\ntype: tasks\nproject: {project_name}\n---\n\n# Tasks — {project_name}\n\n"
        );
        write_file(path, &header)?;
    }
    Ok(())
}

/// Tambah task ke project saat ini.
pub fn add(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        anyhow::bail!("judul task kosong. Pakai: v task \"judul\"");
    }
    let p = project::require()?;
    let file = p.vault_tasks_file();
    ensure_header(&file, &p.name)?;
    let date = Local::now().format("%Y-%m-%d").to_string();
    let line = format!("- [ ] {title}  ➕ {date}\n");
    append_to_file(&file, &line)?;
    println!("✓ task ditambahkan ke {}: {title}", p.name);
    Ok(())
}

/// Tandai task ke-N (dihitung dari task belum selesai, urut muncul) sebagai selesai.
pub fn done(number: usize) -> Result<()> {
    let p = project::require()?;
    let file = p.vault_tasks_file();
    if !file.exists() {
        anyhow::bail!("belum ada task untuk project {}", p.name);
    }
    let content = std::fs::read_to_string(&file)?;
    let mut open_idx = 0usize;
    let mut done_line: Option<String> = None;
    let date = Local::now().format("%Y-%m-%d").to_string();
    let new: String = content
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("- [ ]") {
                open_idx += 1;
                if open_idx == number {
                    let marked = line.replacen("- [ ]", "- [x]", 1);
                    let marked = format!("{marked}  ✅ {date}");
                    done_line = Some(marked.clone());
                    return marked;
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    match done_line {
        Some(l) => {
            let new = if content.ends_with('\n') { format!("{new}\n") } else { new };
            write_file(&file, &new)?;
            println!("✓ selesai: {}", l.trim());
        }
        None => anyhow::bail!("task nomor {number} tidak ditemukan (lihat `v tasks --here`)"),
    }
    Ok(())
}

/// Daftar task aktif. `here=true` hanya project saat ini; else semua di 10_Projects.
pub fn list(here: bool) -> Result<()> {
    if here {
        let p = project::require()?;
        print_tasks_file(&p.vault_tasks_file(), &p.name)?;
        return Ok(());
    }
    // semua project
    let projects_dir = vault_root().join("10_Projects");
    if !projects_dir.exists() {
        println!("(belum ada 10_Projects)");
        return Ok(());
    }
    let mut any = false;
    for entry in walk_md(&projects_dir) {
        if entry.file_name().map(|n| n == "tasks.md").unwrap_or(false) {
            let name = entry
                .parent()
                .and_then(|p| p.file_name())
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if print_tasks_file(&entry, &name)? {
                any = true;
            }
        }
    }
    if !any {
        println!("Tidak ada task aktif.");
    }
    Ok(())
}

fn print_tasks_file(file: &std::path::Path, name: &str) -> Result<bool> {
    if !file.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(file)?;
    let open: Vec<&str> = content
        .lines()
        .filter(|l| l.trim_start().starts_with("- [ ]"))
        .collect();
    if open.is_empty() {
        return Ok(false);
    }
    println!("Project: {name}");
    for (i, line) in open.iter().enumerate() {
        let text = line.trim_start().trim_start_matches("- [ ]").trim();
        println!("  [{}] {}", i + 1, text);
    }
    Ok(true)
}

/// Walk sederhana cari file .md (tanpa dependensi eksternal).
fn walk_md(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        if let Ok(rd) = std::fs::read_dir(&dir) {
            for e in rd.flatten() {
                let path = e.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().map(|x| x == "md").unwrap_or(false) {
                    out.push(path);
                }
            }
        }
    }
    out
}
