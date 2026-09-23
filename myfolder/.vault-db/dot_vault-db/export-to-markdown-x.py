#!/usr/bin/env python3
"""
Export SQLite vault to Obsidian-compatible markdown files.
Organizes notes by context/subcategory structure.
"""

import os
import sqlite3
import json
from pathlib import Path
from datetime import datetime

# Config
VAULT_DB = Path.home() / "myfolder" / ".vault-db" / "vault.db"
EXPORT_DIR = Path.home() / "myfolder" / "obsidian-vault"

def slugify(text: str) -> str:
    """Convert text to filesystem-safe slug."""
    return "".join(c if c.isalnum() or c in "-_" else "-" for c in text).strip("-")

def get_notes(conn):
    """Fetch all active notes with metadata."""
    cursor = conn.cursor()
    cursor.execute("""
        SELECT id, title, content, type, category, parent_context, subcategory,
               project_folder, tags, status, priority, created_at, updated_at, created_by
        FROM notes WHERE status='active'
        ORDER BY parent_context, subcategory, title
    """)
    return cursor.fetchall()

def get_links(conn):
    """Fetch all links."""
    cursor = conn.cursor()
    cursor.execute("""
        SELECT from_note_id, to_note_id, link_type FROM links
    """)
    return cursor.fetchall()

def get_attachments(conn):
    """Fetch all attachments."""
    cursor = conn.cursor()
    cursor.execute("""
        SELECT note_id, filename, path, type FROM attachments
    """)
    return cursor.fetchall()

def build_wikilinks(note_id, links, note_map):
    """Generate wikilinks for a note."""
    outbound = [note_map[tid]["title"] for fid, tid, _ in links if fid == note_id]
    inbound = [note_map[fid]["title"] for fid, tid, _ in links if tid == note_id]
    return outbound, inbound

def write_note(note, outbound, inbound, attachments, export_dir):
    """Write a single note as markdown."""
    (note_id, title, content, ntype, category, parent_context, subcategory,
     project_folder, tags, status, priority, created_at, updated_at, created_by) = note
    
    # Determine folder structure
    if parent_context:
        folder = export_dir / "learning" / parent_context
        if subcategory:
            folder = folder / subcategory
    elif category:
        folder = export_dir / category
    else:
        folder = export_dir / "uncategorized"
    
    folder.mkdir(parents=True, exist_ok=True)
    
    # Filename
    safe_title = slugify(title)
    filepath = folder / f"{safe_title}.md"
    
    # Handle duplicates
    counter = 1
    original = filepath
    while filepath.exists():
        filepath = original.parent / f"{original.stem}-{counter}{original.suffix}"
        counter += 1
    
    # Build frontmatter
    frontmatter = {
        "id": note_id,
        "title": title,
        "type": ntype,
        "category": category,
        "status": status,
        "priority": priority,
        "created_at": created_at,
        "updated_at": updated_at,
        "created_by": created_by,
    }
    if parent_context:
        frontmatter["parent_context"] = parent_context
    if subcategory:
        frontmatter["subcategory"] = subcategory
    if project_folder:
        frontmatter["project_folder"] = project_folder
    if tags:
        try:
            frontmatter["tags"] = json.loads(tags)
        except:
            frontmatter["tags"] = tags
    
    # Build markdown
    lines = ["---"]
    for k, v in frontmatter.items():
        if v is not None:
            if isinstance(v, (list, dict)):
                lines.append(f"{k}: {json.dumps(v)}")
            else:
                lines.append(f"{k}: {v}")
    lines.append("---")
    lines.append("")
    
    # Content
    if content:
        lines.append(content)
        lines.append("")
    
    # Links section
    if outbound or inbound:
        lines.append("## Links")
        if outbound:
            lines.append("### Outbound")
            for t in outbound:
                lines.append(f"- [[{t}]]")
        if inbound:
            lines.append("### Inbound")
            for t in inbound:
                lines.append(f"- [[{t}]]")
        lines.append("")
    
    # Attachments
    if attachments:
        lines.append("## Attachments")
        for filename, path, atype in attachments:
            lines.append(f"- {filename} ({atype or 'file'})")
        lines.append("")
    
    filepath.write_text("\n".join(lines), encoding="utf-8")
    return filepath

def main():
    if not VAULT_DB.exists():
        print(f"✗ Database not found: {VAULT_DB}")
        return 1
    
    conn = sqlite3.connect(VAULT_DB)
    conn.row_factory = sqlite3.Row
    
    print(f"→ Exporting from {VAULT_DB}")
    print(f"→ Writing to {EXPORT_DIR}")
    
    # Fetch all data
    notes = get_notes(conn)
    links = get_links(conn)
    attachments = get_attachments(conn)
    
    # Build note map for wikilinks
    note_map = {n["id"]: dict(n) for n in notes}
    
    # Build attachment map
    attach_map = {}
    for note_id, filename, path, atype in attachments:
        attach_map.setdefault(note_id, []).append((filename, path, atype))
    
    # Export each note
    exported = 0
    for note in notes:
        note_id = note["id"]
        outbound, inbound = build_wikilinks(note_id, links, note_map)
        filepath = write_note(
            note, outbound, inbound, attach_map.get(note_id, []), EXPORT_DIR
        )
        print(f"  ✓ {filepath.relative_to(EXPORT_DIR)}")
        exported += 1
    
    conn.close()
    print(f"\n✓ Exported {exported} notes to {EXPORT_DIR}")
    return 0

if __name__ == "__main__":
    exit(main())