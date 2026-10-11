//! Query file-first: search (ripgrep bila ada) dan stats (hitung dari file).

use crate::util::vault_root;
use anyhow::Result;
use std::process::Command;

/// Cari kata kunci di seluruh vault. Pakai ripgrep bila tersedia; fallback internal.
pub fn search(query: &str) -> Result<()> {
    let root = vault_root();
    // Coba ripgrep
    let rg = Command::new("rg")
        .arg("--smart-case")
        .arg("-n")
        .arg("--color=never")
        .arg("-g")
        .arg("*.md")
        .arg(query)
        .arg(&root)
        .output();

    match rg {
        Ok(out) if out.status.success() || out.status.code() == Some(1) => {
            let text = String::from_utf8_lossy(&out.stdout);
            if text.trim().is_empty() {
                println!("Tidak ada hasil untuk: {query}");
            } else {
                // Ringkas path agar relatif terhadap vault
                let root_str = format!("{}/", root.display());
                for line in text.lines() {
                    println!("{}", line.replacen(&root_str, "", 1));
                }
            }
            Ok(())
        }
        _ => fallback_search(query),
    }
}

fn fallback_search(query: &str) -> Result<()> {
    let root = vault_root();
    let needle = query.to_lowercase();
    let mut found = false;
    for path in walk_md(&root) {
        if let Ok(content) = std::fs::read_to_string(&path) {
            for (i, line) in content.lines().enumerate() {
                if line.to_lowercase().contains(&needle) {
                    let rel = path.strip_prefix(&root).unwrap_or(&path);
                    println!("{}:{}:{}", rel.display(), i + 1, line.trim());
                    found = true;
                }
            }
        }
    }
    if !found {
        println!("Tidak ada hasil untuk: {query}");
    }
    Ok(())
}

/// Statistik dihitung langsung dari file markdown.
pub fn stats() -> Result<()> {
    let root = vault_root();
    let files = walk_md(&root);
    let total_notes = files.len();

    let mut open_tasks = 0usize;
    let mut done_tasks = 0usize;
    let mut daily_count = 0usize;
    let mut projects = std::collections::BTreeSet::new();

    for path in &files {
        let rel = path.strip_prefix(&root).unwrap_or(path);
        let rel_str = rel.to_string_lossy();
        if rel_str.starts_with("00_Daily/") {
            daily_count += 1;
        }
        if rel_str.starts_with("10_Projects/") {
            if let Some(seg) = rel_str.strip_prefix("10_Projects/").and_then(|s| s.split('/').next()) {
                projects.insert(seg.to_string());
            }
        }
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                let t = line.trim_start();
                if t.starts_with("- [ ]") {
                    open_tasks += 1;
                } else if t.starts_with("- [x]") || t.starts_with("- [X]") {
                    done_tasks += 1;
                }
            }
        }
    }

    println!("Statistik Vault ({}):", root.display());
    println!("  Total catatan (.md) : {total_notes}");
    println!("  Daily notes         : {daily_count}");
    println!("  Projects            : {}", projects.len());
    println!("  Task aktif          : {open_tasks}");
    println!("  Task selesai        : {done_tasks}");
    if !projects.is_empty() {
        println!("  Daftar project      : {}", projects.into_iter().collect::<Vec<_>>().join(", "));
    }
    Ok(())
}

fn walk_md(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        // lewati .git dan folder tersembunyi berat
        if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
            if name == ".git" || name == "node_modules" || name == "target" {
                continue;
            }
        }
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
