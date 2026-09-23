-- Extended Obsidian SQLite Vault Schema
-- Support untuk sub-context, project linking, multi-device sync

-- Extended notes table
CREATE TABLE IF NOT EXISTS notes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  title TEXT NOT NULL,
  content TEXT,
  type TEXT NOT NULL CHECK (type IN ('daily', 'task', 'learning', 'research', 'reference', 'credential')),
  category TEXT NOT NULL CHECK (category IN ('daily-notes', 'work', 'learning', 'research', 'reference', 'credentials-vault')),
  
  -- NEW: Sub-context support
  parent_context TEXT,              -- "rust-esp32", "fastapi-api", etc
  subcategory TEXT,                 -- "setup", "wifi", "database", etc
  
  -- NEW: Project linking
  project_folder TEXT,              -- "~/myfolder/work/project-a"
  
  -- NEW: Multi-device tracking
  device_origin TEXT DEFAULT 'local',  -- Device yang create: cachy, windows, hp, etc
  
  -- Original fields
  tags TEXT,                        -- JSON array
  status TEXT DEFAULT 'active' CHECK (status IN ('active', 'archived', 'deleted')),
  priority INTEGER DEFAULT 0,       -- -1=low, 0=normal, 1=high
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  created_by TEXT DEFAULT 'manual',
  UNIQUE(title, category, parent_context)
);

-- Extended tasks table
CREATE TABLE IF NOT EXISTS tasks (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  note_id INTEGER NOT NULL UNIQUE,
  due_date DATE,
  completed_at TIMESTAMP,
  assigned_to TEXT,
  -- NEW: Link to project
  project_folder TEXT,
  FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);

-- Context metadata table (NEW)
CREATE TABLE IF NOT EXISTS contexts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  parent_context TEXT NOT NULL UNIQUE,
  category TEXT NOT NULL,
  title TEXT,
  description TEXT,
  project_folder TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  status TEXT DEFAULT 'active'
);

-- Daily notes (unchanged)
CREATE TABLE IF NOT EXISTS daily_notes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  note_id INTEGER NOT NULL UNIQUE,
  date DATE NOT NULL UNIQUE,
  mood TEXT,
  summary TEXT,
  FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);

-- Links/wikilinks (unchanged)
CREATE TABLE IF NOT EXISTS links (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  from_note_id INTEGER NOT NULL,
  to_note_id INTEGER NOT NULL,
  link_type TEXT DEFAULT 'reference',
  FOREIGN KEY (from_note_id) REFERENCES notes(id) ON DELETE CASCADE,
  FOREIGN KEY (to_note_id) REFERENCES notes(id) ON DELETE CASCADE,
  UNIQUE(from_note_id, to_note_id)
);

-- Attachments (unchanged)
CREATE TABLE IF NOT EXISTS attachments (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  note_id INTEGER NOT NULL,
  filename TEXT NOT NULL,
  path TEXT NOT NULL,
  type TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
);

-- Sync metadata (unchanged)
CREATE TABLE IF NOT EXISTS sync_metadata (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  last_sync_at TIMESTAMP,
  device_name TEXT,
  vault_version TEXT
);

-- Device tracking table (NEW)
CREATE TABLE IF NOT EXISTS devices (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  device_name TEXT NOT NULL UNIQUE,
  device_type TEXT,                 -- cachy, windows, hp, etc
  last_sync TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  is_active INTEGER DEFAULT 1
);

-- Project references table (NEW)
CREATE TABLE IF NOT EXISTS projects (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  project_name TEXT NOT NULL,
  project_folder TEXT NOT NULL UNIQUE,
  category TEXT,                    -- work, personal, etc
  description TEXT,
  created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
  last_accessed TIMESTAMP
);

-- Enhanced indexes for fast queries
CREATE INDEX IF NOT EXISTS idx_notes_parent_context ON notes(parent_context);
CREATE INDEX IF NOT EXISTS idx_notes_subcategory ON notes(subcategory);
CREATE INDEX IF NOT EXISTS idx_notes_project_folder ON notes(project_folder);
CREATE INDEX IF NOT EXISTS idx_notes_device_origin ON notes(device_origin);
CREATE INDEX IF NOT EXISTS idx_notes_category ON notes(category);
CREATE INDEX IF NOT EXISTS idx_notes_type ON notes(type);
CREATE INDEX IF NOT EXISTS idx_notes_status ON notes(status);
CREATE INDEX IF NOT EXISTS idx_notes_created_at ON notes(created_at);
CREATE INDEX IF NOT EXISTS idx_contexts_parent ON contexts(parent_context);
CREATE INDEX IF NOT EXISTS idx_tasks_project ON tasks(project_folder);
CREATE INDEX IF NOT EXISTS idx_projects_folder ON projects(project_folder);
