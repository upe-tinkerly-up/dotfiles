//! Sinkronisasi dua arah antara Kanban `Board.md` (plugin obsidian-kanban) dan
//! file task (`10_Projects/<project>/tasks/*.md`).
//!
//! Model kanonik (sumber kebenaran = file task):
//!   status task (frontmatter) -> lane Kanban
//!     todo    -> "📥 Backlog"
//!     active  -> "🔨 In Progress"
//!     done    -> "✅ Done"
//!     archive -> section "Archive"
//!   Tiap SUBTASK menjadi satu card:
//!     - [ ] <teks subtask> [[<task-file-stem>|<judul-pendek>]]
//!   Backlink `[[...]]` memungkinkan pelacakan balik card -> file task.
//!
//! Perintah:
//!   v board push  : file task -> Board.md (file menang), auto-done.
//!   v board pull  : Board.md -> file task (board menang untuk centang/lane).
//!   v board sync  : pull lalu push (rekonsiliasi).

use crate::project::{self, Project};
use crate::util::write_file;
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::PathBuf;

const LANE_BACKLOG: &str = "📥 Backlog";
const LANE_PROGRESS: &str = "🔨 In Progress";
const LANE_DONE: &str = "✅ Done";
const LANE_ARCHIVE: &str = "Archive";

/// status task -> nama lane.
fn status_to_lane(status: &str) -> &'static str {
    match status {
        "active" => LANE_PROGRESS,
        "done" => LANE_DONE,
        "archive" => LANE_ARCHIVE,
        _ => LANE_BACKLOG, // todo & fallback
    }
}

/// nama lane -> status task.
fn lane_to_status(lane: &str) -> &'static str {
    let l = lane.trim();
    if l.contains("In Progress") || l.contains("🔨") {
        "active"
    } else if l.contains("Done") || l.contains("✅") {
        "done"
    } else if l.eq_ignore_ascii_case("archive") {
        "archive"
    } else {
        "todo"
    }
}

// ───────────────────────── Model file task ─────────────────────────

#[derive(Clone)]
struct Subtask {
    text: String,
    done: bool,
}

#[derive(Clone)]
struct TaskFile {
    path: PathBuf,
    stem: String, // nama file tanpa .md (dipakai untuk backlink)
    title: String,
    status: String,
    subs: Vec<Subtask>,
}

impl TaskFile {
    /// judul pendek untuk alias backlink (biar card ringkas).
    fn short(&self) -> String {
        self.title.chars().take(40).collect::<String>()
    }
}

fn tasks_dir(p: &Project) -> PathBuf {
    p.vault_dir().join("tasks")
}

fn board_path(p: &Project) -> PathBuf {
    p.vault_dir().join("Board.md")
}

/// Baca semua file task sebuah project.
fn read_tasks(p: &Project) -> Result<Vec<TaskFile>> {
    let dir = tasks_dir(p);
    let mut out = Vec::new();
    if !dir.exists() {
        return Ok(out);
    }
    let mut entries: Vec<_> = std::fs::read_dir(&dir)?
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
        .collect();
    entries.sort();
    for path in entries {
        let content = std::fs::read_to_string(&path)?;
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let title = fm_value(&content, "title")
            .map(|t| t.trim_matches('"').to_string())
            .unwrap_or_else(|| stem.clone());
        let status = fm_value(&content, "status")
            .map(|s| s.trim_matches('"').to_string())
            .unwrap_or_else(|| "todo".into());
        let subs = parse_subtasks(&content);
        out.push(TaskFile { path, stem, title, status, subs });
    }
    Ok(out)
}

// ───────────────────────── Model Board.md ─────────────────────────

/// Satu card Kanban (baris checkbox) di dalam sebuah lane.
#[derive(Clone)]
struct Card {
    done: bool,
    text: String,       // teks tanpa backlink
    link_stem: Option<String>, // stem file task bila ada backlink [[stem|..]]
    raw_suffix: String, // sisa setelah teks (mis. tanggal @{..}) yang dipertahankan
}

/// Parsing hasil Board.md: lane berurutan + settings block mentah.
struct Board {
    lanes: Vec<(String, Vec<Card>)>, // (judul lane, cards)
    frontmatter: String,             // blok --- ... --- awal (apa adanya)
    settings_block: String,          // blok %% kanban:settings %% apa adanya (termasuk ***)
}

/// Ambil nilai frontmatter sederhana.
fn fm_value(content: &str, key: &str) -> Option<String> {
    let mut lines = content.lines();
    // frontmatter kanban bisa diawali baris kosong setelah ---; toleran.
    let first = lines.next()?;
    if first.trim() != "---" {
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

/// Parse baris card Kanban: `- [ ] teks [[stem|alias]] @{date}`.
fn parse_card_line(line: &str) -> Option<Card> {
    let t = line.trim_start();
    let rest = t.strip_prefix("- [").or_else(|| t.strip_prefix("* ["))?;
    let mark = rest.chars().next()?;
    let after = rest[1..].strip_prefix(']')?.trim_start();
    let done = matches!(mark, 'x' | 'X');

    // Ekstrak backlink pertama [[stem|alias]] atau [[stem]]
    let mut link_stem = None;
    let mut text = after.to_string();
    let mut raw_suffix = String::new();
    if let Some(start) = after.find("[[") {
        if let Some(end_rel) = after[start..].find("]]") {
            let inner = &after[start + 2..start + end_rel];
            let stem = inner.split('|').next().unwrap_or(inner).trim().to_string();
            link_stem = Some(stem);
            // teks = sebelum backlink; suffix = sesudah backlink
            text = after[..start].trim().to_string();
            raw_suffix = after[start + end_rel + 2..].trim_end().to_string();
        }
    } else {
        // tak ada backlink: pisahkan trailing @{..} sebagai suffix bila ada
        if let Some(at) = after.find("@{") {
            text = after[..at].trim().to_string();
            raw_suffix = after[at..].trim_end().to_string();
        }
    }
    Some(Card { done, text, link_stem, raw_suffix })
}

/// Parse Board.md menjadi struktur.
fn parse_board(content: &str) -> Board {
    // 1) frontmatter awal (--- .. ---), termasuk baris kosong di dalamnya.
    let mut fm = String::new();
    let mut body_start = 0usize;
    let lines: Vec<&str> = content.lines().collect();
    if lines.first().map(|l| l.trim()) == Some("---") {
        fm.push_str("---\n");
        let mut i = 1;
        while i < lines.len() {
            fm.push_str(lines[i]);
            fm.push('\n');
            if lines[i].trim() == "---" {
                i += 1;
                break;
            }
            i += 1;
        }
        body_start = i;
    }

    // 2) settings block: dari baris yang mengandari "%% kanban:settings" sampai akhir.
    //    Juga tangkap separator '***' sebelum Archive bila ada (dipertahankan di tempatnya).
    let mut settings_block = String::new();
    let mut body_end = lines.len();
    for (i, l) in lines.iter().enumerate().skip(body_start) {
        if l.trim_start().starts_with("%% kanban:settings") {
            body_end = i;
            settings_block = lines[i..].join("\n");
            break;
        }
    }

    // 3) parse lane di antara body_start..body_end
    let mut lanes: Vec<(String, Vec<Card>)> = Vec::new();
    let mut cur: Option<(String, Vec<Card>)> = None;
    for l in &lines[body_start..body_end] {
        let t = l.trim_start();
        if let Some(h) = t.strip_prefix("## ") {
            if let Some(prev) = cur.take() {
                lanes.push(prev);
            }
            cur = Some((h.trim().to_string(), Vec::new()));
        } else if t == "***" {
            // separator sebelum Archive — abaikan sebagai konten (diregenerasi).
            continue;
        } else if let Some(card) = parse_card_line(l) {
            if let Some((_, cards)) = cur.as_mut() {
                cards.push(card);
            }
        }
    }
    if let Some(prev) = cur.take() {
        lanes.push(prev);
    }

    Board { lanes, frontmatter: fm, settings_block }
}

/// Render Board kembali ke teks, mempertahankan frontmatter & settings block.
fn render_board(board: &Board) -> String {
    let mut s = String::new();
    if !board.frontmatter.is_empty() {
        s.push_str(&board.frontmatter);
        s.push('\n');
    }
    let mut first = true;
    for (lane, cards) in &board.lanes {
        if !first {
            s.push('\n');
        }
        first = false;
        // Archive didahului separator *** (konvensi Kanban).
        if lane.eq_ignore_ascii_case(LANE_ARCHIVE) {
            s.push_str("***\n\n");
        }
        s.push_str(&format!("## {lane}\n\n"));
        for c in cards {
            s.push_str(&render_card(c));
            s.push('\n');
        }
        s.push('\n');
    }
    if !board.settings_block.is_empty() {
        s.push_str("\n");
        s.push_str(&board.settings_block);
        if !board.settings_block.ends_with('\n') {
            s.push('\n');
        }
    }
    s
}

fn render_card(c: &Card) -> String {
    let mark = if c.done { 'x' } else { ' ' };
    let mut line = format!("- [{mark}] {}", c.text);
    if let Some(stem) = &c.link_stem {
        line.push_str(&format!(" [[{stem}]]"));
    }
    if !c.raw_suffix.is_empty() {
        line.push(' ');
        line.push_str(&c.raw_suffix);
    }
    line
}

// ───────────────────────── Subtask parse (file task) ─────────────────────────

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
        if let Some((done, text)) = parse_checkbox(line) {
            if !text.is_empty() {
                out.push(Subtask { text, done });
            }
        }
    }
    out
}

fn parse_checkbox(line: &str) -> Option<(bool, String)> {
    let t = line.trim_start();
    let t = t.strip_prefix('>').map(|s| s.trim_start()).unwrap_or(t);
    let t = t.strip_prefix("- [").or_else(|| t.strip_prefix("* ["))?;
    let mark = t.chars().next()?;
    let rest = t[1..].strip_prefix(']')?;
    Some((matches!(mark, 'x' | 'X'), rest.trim().to_string()))
}

// ───────────────────────── PUSH: file task -> Board.md ─────────────────────────

/// Auto-done: bila semua subtask selesai (dan ada subtask), paksa status done.
fn effective_status(t: &TaskFile) -> String {
    if !t.subs.is_empty() && t.subs.iter().all(|s| s.done) {
        "done".to_string()
    } else if t.status == "done" && t.subs.iter().any(|s| !s.done) {
        // kalau ditandai done tapi ada subtask belum selesai, turunkan ke active.
        "active".to_string()
    } else {
        t.status.clone()
    }
}

pub fn push() -> Result<()> {
    let p = project::require()?;
    let tasks = read_tasks(&p)?;
    if tasks.is_empty() {
        anyhow::bail!("tidak ada file task di {}", tasks_dir(&p).display());
    }

    // 1) Terapkan auto-done ke file task (tulis balik status bila berubah).
    let mut updated = 0;
    let mut tasks_eff: Vec<TaskFile> = Vec::new();
    for mut t in tasks {
        let eff = effective_status(&t);
        if eff != t.status {
            let content = std::fs::read_to_string(&t.path)?;
            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            let new = set_fm_status(&content, &eff, &date);
            write_file(&t.path, &new)?;
            t.status = eff;
            updated += 1;
        }
        tasks_eff.push(t);
    }

    // 2) Bangun card per lane dari subtask (card ikut status task induk).
    let mut by_lane: BTreeMap<&str, Vec<Card>> = BTreeMap::new();
    for lane in [LANE_BACKLOG, LANE_PROGRESS, LANE_DONE, LANE_ARCHIVE] {
        by_lane.insert(lane, Vec::new());
    }
    for t in &tasks_eff {
        let lane = status_to_lane(&t.status);
        if t.subs.is_empty() {
            // task tanpa subtask tetap muncul sbg satu card (checkbox = status done).
            by_lane.get_mut(lane).unwrap().push(Card {
                done: t.status == "done",
                text: t.title.clone(),
                link_stem: Some(format!("{}|{}", t.stem, t.short())),
                raw_suffix: String::new(),
            });
        } else {
            for s in &t.subs {
                by_lane.get_mut(lane).unwrap().push(Card {
                    done: s.done,
                    text: s.text.clone(),
                    link_stem: Some(format!("{}|{}", t.stem, t.short())),
                    raw_suffix: String::new(),
                });
            }
        }
    }

    // 3) Muat board lama untuk mempertahankan frontmatter, settings, dan
    //    card manual (tanpa backlink) yang tak dikelola v.
    let bpath = board_path(&p);
    let old = if bpath.exists() {
        parse_board(&std::fs::read_to_string(&bpath)?)
    } else {
        default_board(&p.name)
    };

    // Pertahankan card manual (tanpa backlink) di lane masing-masing.
    let mut manual: BTreeMap<String, Vec<Card>> = BTreeMap::new();
    for (lane, cards) in &old.lanes {
        let keep: Vec<Card> = cards.iter().filter(|c| c.link_stem.is_none()).cloned().collect();
        if !keep.is_empty() {
            manual.insert(lane.clone(), keep);
        }
    }

    // 4) Susun lane final dalam urutan tetap.
    let order = [LANE_BACKLOG, LANE_PROGRESS, LANE_DONE, LANE_ARCHIVE];
    let mut lanes: Vec<(String, Vec<Card>)> = Vec::new();
    for lane in order {
        let mut cards = Vec::new();
        if let Some(m) = manual.get(lane) {
            cards.extend(m.clone());
        }
        cards.extend(by_lane.get(lane).cloned().unwrap_or_default());
        lanes.push((lane.to_string(), cards));
    }

    let board = Board { lanes, frontmatter: old.frontmatter, settings_block: old.settings_block };
    write_file(&bpath, &render_board(&board))?;
    println!(
        "✓ board push: {} task → {} ({} status auto-update)",
        tasks_eff.len(),
        bpath.display(),
        updated
    );
    Ok(())
}

// ───────────────────────── PULL: Board.md -> file task ─────────────────────────

pub fn pull() -> Result<()> {
    let p = project::require()?;
    let bpath = board_path(&p);
    if !bpath.exists() {
        anyhow::bail!("belum ada Board.md di {} — jalankan `v board push` dulu", bpath.display());
    }
    let board = parse_board(&std::fs::read_to_string(&bpath)?);
    let tasks = read_tasks(&p)?;
    let mut task_by_stem: BTreeMap<String, TaskFile> =
        tasks.into_iter().map(|t| (t.stem.clone(), t)).collect();

    // Kumpulkan: untuk tiap stem, lane terakhir yang memuatnya + status centang per teks subtask.
    // (board menang untuk centang & lane)
    let mut lane_of: BTreeMap<String, String> = BTreeMap::new();
    let mut check_of: BTreeMap<String, Vec<(String, bool)>> = BTreeMap::new();
    for (lane, cards) in &board.lanes {
        for c in cards {
            if let Some(stem) = card_stem(c) {
                lane_of.insert(stem.clone(), lane.clone());
                check_of.entry(stem).or_default().push((c.text.clone(), c.done));
            }
        }
    }

    let mut changed = 0;
    for (stem, new_lane) in &lane_of {
        let Some(t) = task_by_stem.get_mut(stem) else { continue };
        let content = std::fs::read_to_string(&t.path)?;
        let mut new = content.clone();

        // a) update centang subtask sesuai board (match by teks).
        if let Some(checks) = check_of.get(stem) {
            for (text, done) in checks {
                new = set_subtask_done_by_text(&new, text, *done);
            }
        }

        // b) update status dari lane (board menang untuk perpindahan lane).
        let board_status = lane_to_status(new_lane);
        // tapi hormati auto-done bila semua subtask jadi selesai setelah update.
        let subs_now = parse_subtasks(&new);
        let final_status = if !subs_now.is_empty() && subs_now.iter().all(|s| s.done) {
            "done"
        } else {
            board_status
        };
        if fm_value(&new, "status").map(|s| s.trim_matches('"').to_string()).as_deref() != Some(final_status) {
            let date = chrono::Local::now().format("%Y-%m-%d").to_string();
            new = set_fm_status(&new, final_status, &date);
        }

        if new != content {
            write_file(&t.path, &new)?;
            changed += 1;
        }
    }
    println!("✓ board pull: {changed} file task diperbarui dari {}", bpath.display());
    Ok(())
}

pub fn sync() -> Result<()> {
    pull()?;
    push()?;
    println!("✓ board sync selesai (pull → push).");
    Ok(())
}

/// Ambil stem dari card (buang alias setelah '|').
fn card_stem(c: &Card) -> Option<String> {
    c.link_stem
        .as_ref()
        .map(|s| s.split('|').next().unwrap_or(s).trim().to_string())
}

/// Board kosong default untuk project baru.
fn default_board(project: &str) -> Board {
    let fm = format!(
        "---\n\nkanban-plugin: board\ntype: kanban\nproject: {project}\ntags:\n  - kanban\n  - {project}\n\n---\n"
    );
    let settings = "%% kanban:settings\n```\n{\"kanban-plugin\":\"board\",\"show-relative-date\":true}\n```\n%%".to_string();
    Board {
        lanes: vec![
            (LANE_BACKLOG.into(), vec![]),
            (LANE_PROGRESS.into(), vec![]),
            (LANE_DONE.into(), vec![]),
            (LANE_ARCHIVE.into(), vec![]),
        ],
        frontmatter: fm,
        settings_block: settings,
    }
}

// ───────────────────────── helper tulis file task ─────────────────────────

/// Set checkbox subtask (match persis teks) ke done/undone dalam section Subtask.
fn set_subtask_done_by_text(content: &str, text: &str, done: bool) -> String {
    let mut in_section = false;
    let mut out: Vec<String> = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("## ") {
            in_section = trimmed[3..].trim().eq_ignore_ascii_case("Subtask");
            out.push(line.to_string());
            continue;
        }
        if in_section {
            if let Some((_, t)) = parse_checkbox(line) {
                if t == text {
                    out.push(replace_mark(line, if done { 'x' } else { ' ' }));
                    continue;
                }
            }
        }
        out.push(line.to_string());
    }
    let mut s = out.join("\n");
    if content.ends_with('\n') {
        s.push('\n');
    }
    s
}

fn replace_mark(line: &str, mark: char) -> String {
    if let Some(open) = line.find("- [") {
        let after = open + 3;
        if line.len() > after {
            let mut s = String::with_capacity(line.len());
            s.push_str(&line[..after]);
            s.push(mark);
            s.push_str(&line[after + 1..]);
            return s;
        }
    }
    line.to_string()
}

/// Set/ubah `status:` di frontmatter + `updated:`.
fn set_fm_status(content: &str, status: &str, date: &str) -> String {
    let mut out = String::new();
    let mut in_fm = false;
    let mut closed = false;
    let mut status_written = false;
    let mut updated_written = false;
    for (idx, line) in content.lines().enumerate() {
        if idx == 0 && line.trim() == "---" {
            in_fm = true;
            out.push_str(line);
            out.push('\n');
            continue;
        }
        if in_fm && !closed && line.trim() == "---" {
            if !status_written {
                out.push_str(&format!("status: {status}\n"));
            }
            if !updated_written {
                out.push_str(&format!("updated: \"{date}\"\n"));
            }
            out.push_str(line);
            out.push('\n');
            closed = true;
            in_fm = false;
            continue;
        }
        if in_fm && line.trim_start().starts_with("status:") {
            out.push_str(&format!("status: {status}\n"));
            status_written = true;
            continue;
        }
        if in_fm && line.trim_start().starts_with("updated:") {
            out.push_str(&format!("updated: \"{date}\"\n"));
            updated_written = true;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
