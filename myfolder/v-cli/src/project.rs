//! Deteksi project dari direktori kerja dan pemetaan ke lokasi vault.

use crate::util::{slugify, vault_root};
use std::path::PathBuf;

pub struct Project {
    /// Nama project (= nama folder root project).
    pub name: String,
    /// Folder kode project (root yang mengandung .git).
    pub code_dir: PathBuf,
}

impl Project {
    /// Folder project di vault PARA: 10_Projects/<nama>/
    pub fn vault_dir(&self) -> PathBuf {
        vault_root().join("10_Projects").join(slugify(&self.name))
    }

    /// File konteks mirror di vault.
    pub fn vault_context_file(&self) -> PathBuf {
        self.vault_dir().join("_context.md")
    }

    /// AGENTS.md di folder kode (sumber kebenaran konteks AI).
    pub fn agents_file(&self) -> PathBuf {
        self.code_dir.join("AGENTS.md")
    }
}

/// Deteksi project dengan menaiki direktori mencari `.git`.
/// Kembalikan None bila tidak ditemukan.
pub fn detect() -> Option<Project> {
    let cwd = std::env::current_dir().ok()?;
    let mut dir = cwd.as_path();
    loop {
        if dir.join(".git").exists() {
            let name = dir.file_name()?.to_string_lossy().to_string();
            return Some(Project {
                name,
                code_dir: dir.to_path_buf(),
            });
        }
        dir = dir.parent()?;
    }
}

/// Deteksi project atau error dengan pesan jelas.
pub fn require() -> anyhow::Result<Project> {
    detect().ok_or_else(|| {
        anyhow::anyhow!(
            "tidak terdeteksi project (tidak ada .git di direktori ini atau induknya).\n\
             Jalankan dari dalam folder project kode, mis. ~/myfolder/work/<project>."
        )
    })
}
