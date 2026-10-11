//! Task per-project: 1 task = 1 file di 10_Projects/<nama>/tasks/<slug>.md,
//! dibuat dari template `task.md`. Status dilacak via frontmatter `status:`.

use crate::project::{self, Project};
use crate::util::{load_template, slugify, vault_root, write_file};
use anyhow::Result;
use chrono::Local;
use std::collections::HashMap;

fn tasks_dir(p: &Project) -> std::path::PathBuf {
    p.vault_dir().join("tasks")
}

/// Tambah task sebagai file dari template task.md.
pub fn add(title: &str) -> Result<()> {
    if title.trim().is_empty() {
        anyhow::bail!("judul task kosong. Pakai: v task \"judul\"");
    }
    let p = project::require()?;
    let slug = slugify(title);
    let file = tasks_dir(&p).join(format!("{slug}.md"));
    if file.exists() {
        anyhow::bail!("task '{title}' sudah ada: {}", file.display());
    }
    let date = Local::now().format("%Y-%m-%d").to_string();
    let mut vars = HashMap::new();
    vars.insert("TITLE", title.to_string());
    vars.insert("PROJECT", p.name.clone());
    vars.insert("DATE", date.clone());
    vars.insert("CONTENT", String::new());
    // kompat placeholder lama
    vars.insert("CONTEXT", p.name.clone());
    vars.insert("SUBCATEGORY", String::new());

    let body = load_template("task", &vars).unwrap_or_else(|| {
        format!("---\ntitle: \"{title}\"\ntype: task\nproject: {}\nstatus: todo\ncreated: {date}\n---\n\n# {title}\n", p.name)
    });
    write_file(&file, &body)?;
    println!("✓ task dibuat: {} ({})", title, file.display());
    Ok(())
}

/// Tandai task ke-N (dari daftar task aktif project ini) selesai:
/// ubah frontmatter `status:` -> done.
pub fn done(number: usize) -> Result<()> {
    let p = project::require()?;
    let active = active_tasks(&tasks_dir(&p))?;
    let (path, title) = active
        .get(number - 1)
        .ok_or_else(|| anyhow::anyhow!("task nomor {number} tidak ditemukan (lihat `v tasks --here`)"))?;
    let content = std::fs::read_to_string(path)?;
    let date = Local::now().format("%Y-%m-%d").to_string();
    let new = set_frontmatter_status(&content, "done", &date);
    write_file(path, &new)?;
    println!("✓ selesai: {title}");
    Ok(())
}

/// Daftar task aktif. `here` = hanya project saat ini; else semua project.
pub fn list(here: bool) -> Result<()> {
    if here {
        let p = project::require()?;
        let active = active_tasks(&tasks_dir(&p))?;
        print_list(&p.name, &active);
        return Ok(());
    }
    let projects_dir = vault_root().join("10_Projects");
    if !projects_dir.exists() {
        println!("(belum ada 10_Projects)");
        return Ok(());
    }
    let mut any = false;
    if let Ok(rd) = std::fs::read_dir(&projects_dir) {
        let mut dirs: Vec<_> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
        dirs.sort();
        for d in dirs {
            let tdir = d.join("tasks");
            let active = active_tasks(&tdir)?;
            if !active.is_empty() {
                let name = d.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                print_list(&name, &active);
                any = true;
            }
        }
    }
    if !any {
        println!("Tidak ada task aktif.");
    }
    Ok(())
}

fn print_list(project: &str, active: &[(std::path::PathBuf, String)]) {
    if active.is_empty() {
        return;
    }
    println!("Project: {project}");
    for (i, (_, title)) in active.iter().enumerate() {
        println!("  [{}] {}", i + 1, title);
    }
}

/// Kumpulkan task aktif (status != done) dari sebuah folder tasks.
fn active_tasks(dir: &std::path::Path) -> Result<Vec<(std::path::PathBuf, String)>> {
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    let mut entries: Vec<_> = std::fs::read_dir(dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
        .collect();
    entries.sort();
    for path in entries {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        let status = frontmatter_value(&content, "status").unwrap_or_else(|| "todo".into());
        if status != "done" {
            let title = frontmatter_value(&content, "title")
                .map(|t| t.trim_matches('"').to_string())
                .unwrap_or_else(|| path.file_stem().unwrap().to_string_lossy().to_string());
            out.push((path, title));
        }
    }
    Ok(out)
}

/// Ambil nilai field frontmatter sederhana (key: value) dari blok YAML pertama.
fn frontmatter_value(content: &str, key: &str) -> Option<String> {
    let mut lines = content.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        if let Some(rest) = line.strip_prefix(&format!("{key}:")) {
            return Some(rest.trim().to_string());
        }
    }
    None
}

/// Set/ubah `status:` di frontmatter dan tambah `completed:` bila done.
fn set_frontmatter_status(content: &str, status: &str, date: &str) -> String {
    let mut out = String::new();
    let mut in_fm = false;
    let mut done_fm = false;
    let mut status_written = false;
    for (idx, line) in content.lines().enumerate() {
        if idx == 0 && line.trim() == "---" {
            in_fm = true;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_fm && !done_fm && line.trim() == "---" {
            if !status_written {
                out.push_str(&format!("status: {status}\n"));
            }
            out.push_str(&format!("completed: {date}\n"));
            out.push_str(line);
            out.push('\n');
            done_fm = true;
            in_fm = false;
            continue;
        }
        if in_fm && line.starts_with("status:") {
            out.push_str(&format!("status: {status}\n"));
            status_written = true;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
