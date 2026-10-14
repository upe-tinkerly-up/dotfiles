//! Task per-project: 1 task = 1 file di 10_Projects/<nama>/tasks/<slug>.md,
//! dibuat dari template `task.md`. Status dilacak via frontmatter `status:`.
//!
//! Subtask disimpan sebagai checklist di dalam callout Obsidian pada section
//! `## Subtask`:
//! ```
//! ## Subtask
//! > [!todo]- Checklist
//! > - [ ] subtask satu
//! > - [x] subtask dua
//! ```

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
    let date = Local::now().format("%Y-%m-%d").to_string();
    let slug = slugify(title);
    let file = tasks_dir(&p).join(format!("{date}-{slug}.md"));
    if file.exists() {
        anyhow::bail!("task '{title}' sudah ada: {}", file.display());
    }
    let mut vars = HashMap::new();
    vars.insert("TITLE", title.to_string());
    vars.insert("PROJECT", p.name.clone());
    vars.insert("DATE", date.clone());
    vars.insert("CONTENT", String::new());
    // kompat placeholder lama
    vars.insert("CONTEXT", p.name.clone());
    vars.insert("SUBCATEGORY", String::new());

    let body = load_template("task", &vars).unwrap_or_else(|| {
        format!("---\ntitle: \"{title}\"\ntype: task\nproject: \"{proj}\"\nstatus: todo\npriority: 1\ncreated: \"{date}\"\nupdated: \"{date}\"\naliases: []\ntags:\n  - task\n  - {proj}\n---\n\n# {title}\n\n## Deskripsi\n\n\n## Subtask\n> [!todo]- Checklist\n> - [ ] \n\n## Status\n- [ ] Todo\n- [ ] In Progress\n- [ ] Done\n\n## Catatan\n- \n", proj = p.name)
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

/// Resolusi task aktif ke-N -> (path, title) untuk project saat ini.
fn resolve_active(number: usize) -> Result<(std::path::PathBuf, String)> {
    let p = project::require()?;
    let active = active_tasks(&tasks_dir(&p))?;
    active
        .get(number - 1)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("task nomor {number} tidak ditemukan (lihat `v tasks --here`)"))
}

/// Tambah subtask ke task (ke dalam callout `## Subtask`).
///
/// - `number` = Some(N): langsung ke task #N (non-interaktif).
/// - `number` = None: mode interaktif — tampilkan daftar task aktif, minta pilih.
///   Bila hanya ada 1 task, langsung dipakai tanpa bertanya.
/// - `title` kosong: minta judul subtask lewat prompt.
pub fn sub_add(number: Option<usize>, title: &str) -> Result<()> {
    let p = project::require()?;
    let active = active_tasks(&tasks_dir(&p))?;
    if active.is_empty() {
        anyhow::bail!("belum ada task aktif di project '{}'. Buat dulu: v task \"judul\"", p.name);
    }

    // Tentukan task target.
    let idx = match number {
        Some(n) => {
            if n == 0 || n > active.len() {
                anyhow::bail!("task nomor {n} tidak ada (lihat `v tasks --here`)");
            }
            n - 1
        }
        None => {
            if active.len() == 1 {
                0
            } else {
                // Interaktif: tampilkan daftar lalu minta pilih.
                println!("Pilih task di project '{}':", p.name);
                for (i, (path, t)) in active.iter().enumerate() {
                    let subs = parse_subtasks(&std::fs::read_to_string(path).unwrap_or_default());
                    let done = subs.iter().filter(|s| s.done).count();
                    if subs.is_empty() {
                        println!("  [{}] {}", i + 1, t);
                    } else {
                        println!("  [{}] {}  ({}/{})", i + 1, t, done, subs.len());
                    }
                }
                let n = prompt_number(&format!("Nomor task [1-{}]: ", active.len()), active.len())?;
                n - 1
            }
        }
    };

    let (path, task_title) = active[idx].clone();

    // Tentukan judul subtask (argumen atau prompt interaktif).
    let sub_title = if title.trim().is_empty() {
        prompt_line("Judul subtask: ")?
    } else {
        title.trim().to_string()
    };
    if sub_title.trim().is_empty() {
        anyhow::bail!("judul subtask kosong — dibatalkan.");
    }

    let content = std::fs::read_to_string(&path)?;
    let new = insert_subtask(&content, sub_title.trim());
    write_file(&path, &new)?;
    println!(
        "✓ subtask ditambah ke [{}] {task_title}: {sub_title}",
        idx + 1
    );
    Ok(())
}

/// Baca satu baris dari stdin (dengan prompt), trim newline.
fn prompt_line(prompt: &str) -> Result<String> {
    use std::io::Write;
    print!("{prompt}");
    std::io::stdout().flush().ok();
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}

/// Minta angka 1..=max dari stdin, ulangi bila tidak valid.
fn prompt_number(prompt: &str, max: usize) -> Result<usize> {
    for _ in 0..5 {
        let s = prompt_line(prompt)?;
        match s.parse::<usize>() {
            Ok(n) if n >= 1 && n <= max => return Ok(n),
            _ => eprintln!("  (masukkan angka 1-{max})"),
        }
    }
    anyhow::bail!("input tidak valid — dibatalkan.")
}

/// Centang subtask ke-M pada task #N (- [ ] -> - [x]).
pub fn sub_done(number: usize, sub: usize) -> Result<()> {
    let (path, task_title) = resolve_active(number)?;
    let content = std::fs::read_to_string(&path)?;
    let (new, done_title) = toggle_subtask(&content, sub, true)?;
    write_file(&path, &new)?;
    println!("✓ subtask selesai [{number}.{sub}] {done_title}  (task: {task_title})");
    Ok(())
}

/// Tampilkan detail task #N + daftar subtask + progress.
pub fn show(number: usize) -> Result<()> {
    let (path, task_title) = resolve_active(number)?;
    let content = std::fs::read_to_string(&path)?;
    let subs = parse_subtasks(&content);
    let done = subs.iter().filter(|s| s.done).count();
    println!("[{number}] {task_title}");
    println!("    file: {}", path.display());
    if subs.is_empty() {
        println!("    (belum ada subtask) — tambah: v task sub {number} \"judul\"");
        return Ok(());
    }
    println!("    subtask ({done}/{}):", subs.len());
    for (i, s) in subs.iter().enumerate() {
        let mark = if s.done { "x" } else { " " };
        println!("    {:>2}. [{}] {}", i + 1, mark, s.text);
    }
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
    for (i, (path, title)) in active.iter().enumerate() {
        let content = std::fs::read_to_string(path).unwrap_or_default();
        let subs = parse_subtasks(&content);
        if subs.is_empty() {
            println!("  [{}] {}", i + 1, title);
        } else {
            let done = subs.iter().filter(|s| s.done).count();
            println!("  [{}] {}  ({}/{})", i + 1, title, done, subs.len());
        }
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

// ───────────────────────── Subtask (callout-aware) ─────────────────────────

/// Satu subtask hasil parse.
struct Subtask {
    text: String,
    done: bool,
}

/// Apakah baris adalah checkbox dalam callout (`> - [ ]` / `> - [x]`) atau
/// checkbox polos (`- [ ]`). Kembalikan (done, teks) bila cocok.
fn parse_checkbox_line(line: &str) -> Option<(bool, String)> {
    let t = line.trim_start();
    // buang penanda callout '>' bila ada
    let t = t.strip_prefix('>').map(|s| s.trim_start()).unwrap_or(t);
    let t = t.strip_prefix("- [").or_else(|| t.strip_prefix("* ["))?;
    // t sekarang mulai dengan 'x] ...' atau ' ] ...'
    let mark = t.chars().next()?;
    let rest = &t[1..];
    let rest = rest.strip_prefix(']')?;
    let done = matches!(mark, 'x' | 'X');
    Some((done, rest.trim().to_string()))
}

/// Parse semua subtask (checkbox non-kosong) di dalam section `## Subtask`.
fn parse_subtasks(content: &str) -> Vec<Subtask> {
    let mut out = Vec::new();
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") {
            in_section = trimmed[3..].trim().eq_ignore_ascii_case("Subtask");
            continue;
        }
        if !in_section {
            continue;
        }
        if let Some((done, text)) = parse_checkbox_line(line) {
            if !text.is_empty() {
                out.push(Subtask { text, done });
            }
        }
    }
    out
}

/// Deteksi gaya callout dari baris checkbox pertama di section (true = pakai `> `).
fn section_uses_callout(content: &str) -> bool {
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") {
            in_section = trimmed[3..].trim().eq_ignore_ascii_case("Subtask");
            continue;
        }
        if in_section && (trimmed.starts_with("> [!") || trimmed.starts_with('>')) {
            return true;
        }
    }
    // default: pakai callout (sesuai template)
    true
}

/// Sisipkan subtask baru ke dalam section `## Subtask`.
/// - Bila section belum ada: buat section + callout + subtask.
/// - Bila ada: sisipkan setelah baris checkbox non-kosong terakhir (atau setelah
///   header callout bila belum ada checkbox terisi), buang placeholder kosong.
fn insert_subtask(content: &str, title: &str) -> String {
    let callout = section_uses_callout(content);
    let new_line = if callout {
        format!("> - [ ] {title}")
    } else {
        format!("- [ ] {title}")
    };

    let lines: Vec<&str> = content.lines().collect();

    // cari batas section ## Subtask
    let mut sec_start: Option<usize> = None; // index baris "## Subtask"
    for (i, l) in lines.iter().enumerate() {
        let t = l.trim_start();
        if t.starts_with("## ") && t[3..].trim().eq_ignore_ascii_case("Subtask") {
            sec_start = Some(i);
            break;
        }
    }

    // Section belum ada → tambahkan di akhir file.
    let Some(sec_start) = sec_start else {
        let mut out = content.trim_end().to_string();
        out.push_str("\n\n## Subtask\n> [!todo]- Checklist\n");
        out.push_str(&new_line);
        out.push('\n');
        return out;
    };

    // akhir section = baris "## " berikutnya atau EOF
    let mut sec_end = lines.len();
    for (i, l) in lines.iter().enumerate().skip(sec_start + 1) {
        if l.trim_start().starts_with("## ") {
            sec_end = i;
            break;
        }
    }

    // di dalam [sec_start+1, sec_end): cari posisi sisip = setelah checkbox
    // terisi terakhir; kalau tak ada, setelah baris callout header `> [!...]`.
    let mut insert_at = sec_start + 1;
    let mut last_header = None;
    for i in (sec_start + 1)..sec_end {
        let t = lines[i].trim_start();
        if t.starts_with("> [!") {
            last_header = Some(i);
        }
        if let Some((_, text)) = parse_checkbox_line(lines[i]) {
            if !text.is_empty() {
                insert_at = i + 1;
            }
        }
    }
    if insert_at == sec_start + 1 {
        // belum ada checkbox terisi → sisip setelah header callout bila ada
        if let Some(h) = last_header {
            insert_at = h + 1;
        }
    }

    // Rebuild: buang baris checkbox KOSONG (placeholder) di dalam section,
    // supaya tidak menumpuk, lalu sisipkan subtask baru.
    let mut out_lines: Vec<String> = Vec::with_capacity(lines.len() + 1);
    for (i, l) in lines.iter().enumerate() {
        if i == insert_at {
            out_lines.push(new_line.clone());
        }
        let in_sec = i > sec_start && i < sec_end;
        if in_sec {
            if let Some((_, text)) = parse_checkbox_line(l) {
                if text.is_empty() {
                    continue; // skip placeholder kosong
                }
            }
        }
        out_lines.push((*l).to_string());
    }
    if insert_at == lines.len() {
        out_lines.push(new_line);
    }

    let mut s = out_lines.join("\n");
    if content.ends_with('\n') {
        s.push('\n');
    }
    s
}

/// Toggle (centang) subtask ke-M (1-based, hanya menghitung checkbox non-kosong).
/// Kembalikan (konten baru, teks subtask).
fn toggle_subtask(content: &str, sub: usize, done: bool) -> Result<(String, String)> {
    let mut in_section = false;
    let mut idx = 0usize;
    let mut found_text: Option<String> = None;
    let mut out_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") {
            in_section = trimmed[3..].trim().eq_ignore_ascii_case("Subtask");
            out_lines.push(line.to_string());
            continue;
        }
        if in_section {
            if let Some((_cur, text)) = parse_checkbox_line(line) {
                if !text.is_empty() {
                    idx += 1;
                    if idx == sub {
                        let mark = if done { 'x' } else { ' ' };
                        // pertahankan indentasi + penanda callout asli
                        let replaced = replace_checkbox_mark(line, mark);
                        found_text = Some(text);
                        out_lines.push(replaced);
                        continue;
                    }
                }
            }
        }
        out_lines.push(line.to_string());
    }

    let text = found_text.ok_or_else(|| {
        anyhow::anyhow!("subtask nomor {sub} tidak ditemukan (lihat `v task show`)")
    })?;
    let mut s = out_lines.join("\n");
    if content.ends_with('\n') {
        s.push('\n');
    }
    Ok((s, text))
}

/// Ganti karakter mark di dalam `[ ]`/`[x]` pada sebuah baris checkbox,
/// mempertahankan prefix (indentasi, `> `, `- `).
fn replace_checkbox_mark(line: &str, mark: char) -> String {
    if let Some(open) = line.find("- [") {
        let after = open + 3; // index char mark
        if line.len() > after {
            let mut s = String::with_capacity(line.len());
            s.push_str(&line[..after]);
            s.push(mark);
            // sisa setelah 1 char mark
            s.push_str(&line[after + mark.len_utf8().min(line.len() - after)..]);
            // Pastikan tetap ada ']' tepat setelah mark (umumnya sudah ada).
            return s;
        }
    }
    line.to_string()
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
