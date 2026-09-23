use std::fs;

use rusqlite::{Connection, params};
use std::path::PathBuf;
use std::env;
use std::process;

struct Vault {
    db_path: PathBuf,
    export_dir: PathBuf,
}

impl Vault {
    fn new() -> Self {
        let home = env::var("HOME").unwrap_or_else(|_| "/home/upe".into());
        let vault_path = env::var("VAULT_PATH")
            .unwrap_or_else(|_| format!("{}/myfolder/.vault-db", home));
        let db_path = PathBuf::from(&vault_path).join("vault.db");
        let export_dir = PathBuf::from(&vault_path).join("../obsidian-vault");
        Vault { db_path, export_dir }
    }

    fn conn(&self) -> Connection {
        Connection::open(&self.db_path).expect("open db")
    }

    fn check_db(&self) {
        if !self.db_path.exists() {
            eprintln!("Database not found at {:?}", self.db_path);
            process::exit(1);
        }
    }
}

fn cmd_list_notes(v: &Vault) {
    v.check_db();
    let conn = v.conn();
    let mut stmt = conn.prepare(
        "SELECT id, title, type, category, parent_context FROM notes WHERE status='active' ORDER BY updated_at DESC LIMIT 10"
    ).unwrap();
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?, r.get::<_, Option<String>>(4)?))
    }).unwrap();
    println!("Recent Notes:");
    for row in rows {
        let (id, title, nt, cat, ctx) = row.unwrap();
        println!("  [{}] {} | {} | {} | {:?}", id, title, nt, cat, ctx);
    }
}

fn cmd_search(v: &Vault, keyword: &str) {
    v.check_db();
    let conn = v.conn();
    let mut stmt = conn.prepare(
        "SELECT id, title, category, parent_context FROM notes WHERE (content LIKE ? OR title LIKE ?) AND status='active' LIMIT 20"
    ).unwrap();
    let kw = format!("%{}%", keyword);
    let rows = stmt.query_map(params![kw.clone(), kw.clone()], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, Option<String>>(3)?))
    }).unwrap();
    println!("Search results for: {}", keyword);
    for row in rows {
        let (id, title, cat, ctx) = row.unwrap();
        println!("  [{}] {} | {} | {:?}", id, title, cat, ctx);
    }
}

fn cmd_tasks(v: &Vault) {
    v.check_db();
    let conn = v.conn();
    let mut stmt = conn.prepare(
        "SELECT n.id, n.title, t.due_date, n.project_folder FROM notes n LEFT JOIN tasks t ON n.id=t.note_id WHERE n.type='task' AND n.status='active' ORDER BY t.due_date, n.updated_at DESC"
    ).unwrap();
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?, r.get::<_, Option<String>>(3)?))
    }).unwrap();
    println!("Active Tasks:");
    for row in rows {
        let (id, title, due, proj) = row.unwrap();
        println!("  [{}] {} | due: {:?} | project: {:?}", id, title, due, proj);
    }
}

fn cmd_stats(v: &Vault) {
    v.check_db();
    let conn = v.conn();
    println!("Vault Statistics:");
    let total: i64 = conn.query_row("SELECT COUNT(*) FROM notes WHERE status='active'", [], |r: &rusqlite::Row| r.get::<_, i64>(0)).unwrap();
    let tasks: i64 = conn.query_row("SELECT COUNT(*) FROM notes WHERE type='task' AND status='active'", [], |r: &rusqlite::Row| r.get::<_, i64>(0)).unwrap();
    let contexts: i64 = conn.query_row("SELECT COUNT(DISTINCT parent_context) FROM notes WHERE type='learning' AND parent_context IS NOT NULL", [], |r: &rusqlite::Row| r.get::<_, i64>(0)).unwrap();
    let projects: i64 = conn.query_row("SELECT COUNT(*) FROM projects", [], |r: &rusqlite::Row| r.get::<_, i64>(0)).unwrap();
    println!("  Total Notes: {}", total);
    println!("  Active Tasks: {}", tasks);
    println!("  Learning Contexts: {}", contexts);
    println!("  Projects: {}", projects);
}

fn cmd_list_contexts(v: &Vault) {
    v.check_db();
    let conn = v.conn();
    let mut stmt = conn.prepare(
        "SELECT parent_context, category, COUNT(*) as notes FROM notes WHERE parent_context IS NOT NULL AND status='active' GROUP BY parent_context ORDER BY parent_context"
    ).unwrap();
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, i64>(2)?))
    }).unwrap();
    println!("Available Contexts:");
    for row in rows {
        let (ctx, cat, count) = row.unwrap();
        println!("  {} | {} | {} notes", ctx, cat, count);
    }
}

fn cmd_context_info(v: &Vault, context: &str) {
    v.check_db();
    let conn = v.conn();
    let mut stmt = conn.prepare(
        "SELECT subcategory, COUNT(*) as notes FROM notes WHERE parent_context=? AND status='active' GROUP BY subcategory"
    ).unwrap();
    let rows = stmt.query_map(params![context], |r| {
        Ok((r.get::<_, Option<String>>(0)?, r.get::<_, i64>(1)?))
    }).unwrap();
    println!("Context: {}", context);
    for row in rows {
        let (sub, count) = row.unwrap();
        println!("  {:?}: {} notes", sub, count);
    }
}

fn cmd_context_switch(v: &Vault, context: &str) {
    let path = v.db_path.parent().unwrap().join(".current_context");
    std::fs::write(&path, context).unwrap();
    println!("Switched to context: {}", context);
}

fn cmd_context_current(v: &Vault) {
    let path = v.db_path.parent().unwrap().join(".current_context");
    if path.exists() {
        println!("{}", std::fs::read_to_string(&path).unwrap());
    } else {
        println!("No context selected");
    }
}

fn cmd_add_note_sub(v: &Vault, title: &str, note_type: &str, category: &str, parent_context: &str, subcategory: &str, content: &str) {
    v.check_db();
    let conn = v.conn();
    let device = env::var("DEVICE_NAME").unwrap_or_else(|_| "local".into());
    let mut stmt = conn.prepare(
        "INSERT INTO notes (title, content, type, category, parent_context, subcategory, device_origin, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, 'cli')"
    ).unwrap();
    stmt.execute(params![title, content, note_type, category, parent_context, subcategory, device]).unwrap();
    let id = conn.last_insert_rowid();
    println!("Note created with ID: {} (context: {}/{})", id, parent_context, subcategory);
}

fn cmd_add_note_project(v: &Vault, title: &str, note_type: &str, category: &str, project_folder: &str, content: &str) {
    v.check_db();
    let conn = v.conn();
    let device = env::var("DEVICE_NAME").unwrap_or_else(|_| "local".into());
    let mut stmt = conn.prepare(
        "INSERT INTO notes (title, content, type, category, project_folder, device_origin) VALUES (?, ?, ?, ?, ?, ?)"
    ).unwrap();
    stmt.execute(params![title, content, note_type, category, project_folder, device]).unwrap();
    let id = conn.last_insert_rowid();
    if note_type == "task" {
        conn.execute("INSERT INTO tasks (note_id, project_folder) VALUES (?, ?)", params![id, project_folder]).unwrap();
    }
    println!("Project note created with ID: {} (linked to: {})", id, project_folder);
}

fn cmd_project_notes(v: &Vault, project_folder: &str) {
    v.check_db();
    let conn = v.conn();
    let mut stmt = conn.prepare(
        "SELECT id, title, type, priority FROM notes WHERE project_folder=? AND status='active' ORDER BY priority DESC, updated_at DESC"
    ).unwrap();
    let rows = stmt.query_map(params![project_folder], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, i32>(3)?))
    }).unwrap();
    println!("Notes for project: {}", project_folder);
    for row in rows {
        let (id, title, nt, prio) = row.unwrap();
        println!("  [{}] {} | {} | priority {}", id, title, nt, prio);
    }
}

fn cmd_config(v: &Vault, action: &str) {
    match action {
        "show" => {
            println!("=== Vault Configuration ===");
            println!("Vault DB: {:?}", v.db_path);
            println!("Vault Export: {:?}", v.export_dir);
        }
        _ => {
            println!("Unknown config action: {}", action);
        }
    }
}

fn cmd_help() {
    println!(r#"vault: Enhanced Obsidian SQLite Vault CLI (Rust)

Basic Commands:
  list-notes (ln)              List recent notes
  search (s) <keyword>         Search notes
  tasks (t)                    Show active tasks
  stats                        Vault statistics
  export (exp)                 Export to markdown
  import (imp)                 Import markdown to DB

Sub-Context Commands:
  add-note-sub (ans) <args>    Add note with context
    Usage: add-note-sub "Title" "type" "category" "context" "subcategory" "content"
  list-contexts (lc)           List all contexts
  context-info (ci) <context>  Show context details
  context-switch (cs) <ctx>    Switch active context
  context-current (cc)         Show current context

Project Commands:
  add-note-project (anp) <args> Add note linked to project
    Usage: add-note-project "Title" "type" "category" "~/path/to/project" "content"
  project-notes (pn) <path>    Show notes for project

Configuration:
  config show                  Show current config

Examples:
  vault ans "Blink LED" "learning" "learning" "rust-esp32" "basics" "..."
  vault s "rust"
  vault lc
  vault cs rust-esp32
  vault pn ~/myfolder/work/project-a"#);
}

fn slugify(s: &str) -> String {
    s.chars().map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '-' }).collect()
}

fn cmd_export(v: &Vault) {
    v.check_db();
    let conn = v.conn();
    let mut stmt = conn.prepare(
        "SELECT id, title, content, type, category, parent_context, subcategory, project_folder, tags, status, priority, created_at, updated_at, created_by FROM notes WHERE status='active'"
    ).unwrap();
    let rows = stmt.query_map([], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, Option<String>>(2)?, r.get::<_, String>(3)?, r.get::<_, String>(4)?, r.get::<_, Option<String>>(5)?, r.get::<_, Option<String>>(6)?, r.get::<_, Option<String>>(7)?, r.get::<_, Option<String>>(8)?, r.get::<_, String>(9)?, r.get::<_, i32>(10)?, r.get::<_, String>(11)?, r.get::<_, String>(12)?, r.get::<_, String>(13)?))
    }).unwrap();
    let mut count = 0;
    for row in rows {
        let (id, title, content, nt, cat, ctx, sub, proj, tags, status, prio, created, updated, created_by) = row.unwrap();
        let folder = if let Some(c) = &ctx {
            let base = v.export_dir.join("learning").join(c);
            if let Some(s) = &sub { base.join(s) } else { base }
        } else {
            v.export_dir.join(&cat)
        };
        fs::create_dir_all(&folder).unwrap();
        let safe = slugify(&title);
        let filepath = folder.join(format!("{}.md", safe));
        let mut md = String::new();
        md.push_str("---\n");
        md.push_str(&format!("id: {}\n", id));
        md.push_str(&format!("title: {}\n", title));
        md.push_str(&format!("type: {}\n", nt));
        md.push_str(&format!("category: {}\n", cat));
        md.push_str(&format!("status: {}\n", status));
        md.push_str(&format!("priority: {}\n", prio));
        md.push_str(&format!("created_at: {}\n", created));
        md.push_str(&format!("updated_at: {}\n", updated));
        if let Some(c) = &ctx { md.push_str(&format!("parent_context: {}\n", c)); }
        if let Some(s) = &sub { md.push_str(&format!("subcategory: {}\n", s)); }
        if let Some(p) = &proj { md.push_str(&format!("project_folder: {}\n", p)); }
        md.push_str("---\n\n");
        if let Some(c) = &content { md.push_str(c); md.push_str("\n\n"); }
        fs::write(&filepath, md).unwrap();
        println!("  {:?}", filepath);
        count += 1;
    }
    println!("Exported {} notes to {:?}", count, v.export_dir);
}

fn cmd_import(v: &Vault) {
    v.check_db();
    let conn = v.conn();
    let mut imported = 0;
    let mut skipped = 0;
    for entry in walkdir::WalkDir::new(&v.export_dir).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() { continue; }
        let path = entry.path();
        if path.extension().map(|e| e != "md").unwrap_or(true) { continue; }
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let (meta, body) = split_frontmatter(&content);
        let title = meta.get("title").cloned().unwrap_or_else(|| path.file_stem().unwrap().to_string_lossy().to_string());
        let nt = meta.get("type").cloned().unwrap_or_else(|| "learning".into());
        let cat = meta.get("category").cloned().unwrap_or_else(|| "learning".into());
        let ctx = meta.get("parent_context").cloned();
        let sub = meta.get("subcategory").cloned();
        let proj = meta.get("project_folder").cloned();
        let id = meta.get("id").and_then(|s| s.parse::<i64>().ok());
        if let Some(existing_id) = id {
            let exists: bool = conn.query_row("SELECT 1 FROM notes WHERE id=?", params![existing_id], |r: &rusqlite::Row| Ok::<_, rusqlite::Error>(true)).unwrap_or(false);
            if exists {
                conn.execute("UPDATE notes SET title=?, content=?, type=?, category=?, parent_context=?, subcategory=?, project_folder=?, updated_at=datetime('now') WHERE id=?", params![title, body, nt, cat, ctx, sub, proj, existing_id]).unwrap();
                skipped += 1;
                continue;
            }
        }
        conn.execute("INSERT INTO notes (title, content, type, category, parent_context, subcategory, project_folder, device_origin, created_by) VALUES (?, ?, ?, ?, ?, ?, ?, 'import', 'cli')", params![title, body, nt, cat, ctx, sub, proj]).unwrap();
        imported += 1;
    }
    println!("Imported {} new, updated {} existing notes", imported, skipped);
}

fn split_frontmatter(s: &str) -> (std::collections::HashMap<String, String>, String) {
    let mut meta = std::collections::HashMap::new();
    let body = if s.starts_with("---\n") {
        let rest = &s[4..];
        if let Some(end) = rest.find("\n---\n") {
            let fm = &rest[..end];
            for line in fm.lines() {
                if let Some(pos) = line.find(':') {
                    let k = line[..pos].trim().to_string();
                    let v = line[pos+1..].trim().to_string();
                    meta.insert(k, v);
                }
            }
            &rest[end+5..]
        } else { s }
    } else { s };
    (meta, body.to_string())
}


fn main() {
    let args: Vec<String> = env::args().collect();
    let vault = Vault::new();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("help");

    match cmd {
        "list-notes" | "ln" => cmd_list_notes(&vault),
        "search" | "s" => cmd_search(&vault, args.get(2).map(|s| s.as_str()).unwrap_or("")),
        "tasks" | "t" => cmd_tasks(&vault),
        "stats" => cmd_stats(&vault),
        "export" | "exp" => cmd_export(&vault),
        "list-contexts" | "lc" => cmd_list_contexts(&vault),
        "context-info" | "ci" => cmd_context_info(&vault, args.get(2).map(|s| s.as_str()).unwrap_or("")),
        "context-switch" | "cs" => cmd_context_switch(&vault, args.get(2).map(|s| s.as_str()).unwrap_or("")),
        "context-current" | "cc" => cmd_context_current(&vault),
        "add-note-sub" | "ans" => cmd_add_note_sub(&vault,
            args.get(2).map(|s| s.as_str()).unwrap_or(""),
            args.get(3).map(|s| s.as_str()).unwrap_or(""),
            args.get(4).map(|s| s.as_str()).unwrap_or(""),
            args.get(5).map(|s| s.as_str()).unwrap_or(""),
            args.get(6).map(|s| s.as_str()).unwrap_or(""),
            args.get(7).map(|s| s.as_str()).unwrap_or("")),
        "add-note-project" | "anp" => cmd_add_note_project(&vault,
            args.get(2).map(|s| s.as_str()).unwrap_or(""),
            args.get(3).map(|s| s.as_str()).unwrap_or(""),
            args.get(4).map(|s| s.as_str()).unwrap_or(""),
            args.get(5).map(|s| s.as_str()).unwrap_or(""),
            args.get(6).map(|s| s.as_str()).unwrap_or("")),
        "project-notes" | "pn" => cmd_project_notes(&vault, args.get(2).map(|s| s.as_str()).unwrap_or("")),
        "import" | "imp" => cmd_import(&vault),
        "config" => cmd_config(&vault, args.get(2).map(|s| s.as_str()).unwrap_or("show")),
        _ => cmd_help(),
    }
}
