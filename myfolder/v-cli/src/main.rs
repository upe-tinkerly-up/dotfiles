//! `v` — File-first PARA vault CLI untuk alur konteks-AI-per-project.
//!
//! Semua perintah menulis langsung sebagai markdown ke struktur PARA di
//! $V_VAULT (default ~/myfolder/obsidian-vault). Tidak ada database.

mod util;
mod project;
mod ctx;
mod task;
mod capture;
mod query;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "v",
    version,
    about = "File-first PARA vault CLI (konteks AI per-project, daily, wiki, task, search)",
    long_about = None
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Konteks AI per-project: tulis AGENTS.md di folder project + mirror ke vault 10_Projects/<nama>/
    Ctx {
        #[command(subcommand)]
        action: Option<CtxAction>,
    },
    /// Task per-project (10_Projects/<nama>/tasks.md)
    Task {
        #[command(subcommand)]
        action: Option<TaskAction>,
        /// Shortcut: `v task "judul"` = tambah task
        title: Option<String>,
    },
    /// Daftar task aktif (semua project atau project saat ini dengan --here)
    Tasks {
        /// Hanya task untuk project di direktori saat ini
        #[arg(long)]
        here: bool,
    },
    /// Daily note: append entri ke 00_Daily/YYYY-MM-DD.md
    Daily {
        /// Isi entri (opsional; tanpa isi hanya memastikan daily note hari ini ada)
        text: Vec<String>,
    },
    /// Halaman wiki: buat/append 30_Resources/LLM_Wiki/<Judul>.md
    Wiki {
        /// Judul halaman
        title: String,
        /// Isi (opsional)
        text: Vec<String>,
    },
    /// Catatan bebas ke path relatif terhadap vault
    Note {
        /// Path relatif dalam vault, mis. 20_Areas/Health/catatan.md
        path: String,
        /// Isi (opsional)
        text: Vec<String>,
    },
    /// Catatan bebas project: 10_Projects/<nama>/notes/<judul>.md (dari dalam folder project)
    Pnote {
        /// Judul catatan
        title: String,
        /// Isi (opsional; tanpa isi hanya membuat file catatan kosong)
        text: Vec<String>,
    },
    /// Daftar catatan bebas untuk project saat ini
    Pnotes,
    /// Cari di seluruh vault (pakai ripgrep bila ada, fallback ke pencarian internal)
    Search {
        /// Kata kunci
        query: String,
    },
    /// Statistik vault (dihitung dari file)
    Stats,
    /// Daftar catatan menurut kategori
    List {
        /// Kategori: daily | wiki | notes | projects (default: daily)
        #[arg(default_value = "daily")]
        kind: String,
    },
    /// Tampilkan path & konfigurasi yang dipakai
    Config,
    /// Panduan penggunaan & workflow lengkap
    Guide,
}

#[derive(Subcommand)]
enum CtxAction {
    /// Tambah entri log bertimestamp ke konteks project
    Log {
        /// Isi log
        text: Vec<String>,
    },
    /// Cetak konteks project (untuk di-feed ke AI)
    Show,
    /// Inisialisasi konteks untuk project saat ini (dipanggil otomatis bila belum ada)
    Init,
    /// Tautkan AGENTS.md ke pintu masuk tool lain (CLAUDE.md, GEMINI.md, .kiro/steering, .hermes) via symlink.
    /// Aman: tidak menimpa file yang sudah ada. Hanya di folder kode (Linux), tidak menyentuh vault.
    Link,
}

#[derive(Subcommand)]
enum TaskAction {
    /// Tambah task baru
    Add {
        /// Judul task
        title: Vec<String>,
    },
    /// Tandai task selesai berdasarkan nomor
    Done {
        /// Nomor task (lihat `v tasks`)
        number: usize,
    },
    /// Tambah subtask ke task nomor N (checklist di dalam callout `## Subtask`)
    Sub {
        /// Nomor task (lihat `v tasks --here`)
        number: usize,
        /// Judul subtask
        title: Vec<String>,
    },
    /// Centang subtask ke-M pada task nomor N
    SubDone {
        /// Nomor task (lihat `v tasks --here`)
        number: usize,
        /// Nomor subtask (lihat `v task show N`)
        sub: usize,
    },
    /// Tampilkan detail task nomor N + daftar subtask + progress
    Show {
        /// Nomor task (lihat `v tasks --here`)
        number: usize,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("v: error: {e:#}");
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Ctx { action } => match action {
            None => ctx::show(),
            Some(CtxAction::Init) => ctx::init().map(|_| ()),
            Some(CtxAction::Show) => ctx::show(),
            Some(CtxAction::Log { text }) => ctx::log(&text.join(" ")),
            Some(CtxAction::Link) => ctx::link(),
        },
        Command::Task { action, title } => match (action, title) {
            (Some(TaskAction::Add { title }), _) => task::add(&title.join(" ")),
            (Some(TaskAction::Done { number }), _) => task::done(number),
            (Some(TaskAction::Sub { number, title }), _) => task::sub_add(number, &title.join(" ")),
            (Some(TaskAction::SubDone { number, sub }), _) => task::sub_done(number, sub),
            (Some(TaskAction::Show { number }), _) => task::show(number),
            (None, Some(t)) => task::add(&t),
            (None, None) => task::list(true),
        },
        Command::Tasks { here } => task::list(here),
        Command::Daily { text } => capture::daily(&text.join(" ")),
        Command::Wiki { title, text } => capture::wiki(&title, &text.join(" ")),
        Command::Note { path, text } => capture::note(&path, &text.join(" ")),
        Command::Pnote { title, text } => capture::pnote(&title, &text.join(" ")),
        Command::Pnotes => capture::pnotes_list(),
        Command::Search { query } => query::search(&query),
        Command::Stats => query::stats(),
        Command::List { kind } => query::list(&kind),
        Command::Config => util::print_config(),
        Command::Guide => {
            util::print_guide();
            Ok(())
        }
    }
}
