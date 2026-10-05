#!/bin/bash
# Backup vault database dan sinkronisasi ke Google Drive

VAULT_DB="/home/upe/myfolder/.vault-db/vault.db"
BACKUP_DIR="/home/upe/backup"
VAULT_DIR="/home/upe/obsidian-vault"

# Buat timestamp
TIMESTAMP=$(date +%Y-%m-%d_%H-%M-%S)

# Backup database
echo "[$(date)] Memulai backup database..."
cp "$VAULT_DB" "$BACKUP_DIR/vault_$TIMESTAMP.db"

# Sinkronisasi ke Google Drive
echo "[$(date)] Sinkronisasi ke Google Drive..."
rclone sync "$VAULT_DIR" gdrive:obsidian-vault/

# Sinkronisasi backup database ke Google Drive
echo "[$(date)] Backup database ke Google Drive..."
rclone sync "$BACKUP_DIR" gdrive:obsidian-vault/backup/

echo "[$(date)] Backup selesai"
