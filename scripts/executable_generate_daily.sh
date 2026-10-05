#!/bin/bash
# Script untuk generate daily note menggunakan template custom

DATE=$(date +%Y-%m-%d)
TIME=$(date +%H:%M:%S)
VAULT_DIR="/home/upe/obsidian-vault"
TEMPLATE_FILE="$VAULT_DIR/templates/daily-standup.md"
TARGET_FILE="$VAULT_DIR/daily-notes/Daily-Note-$DATE.md"

if [ -f "$TARGET_FILE" ]; then
    echo "⚠️  Daily note untuk hari ini sudah ada: $TARGET_FILE"
    exit 0
fi

if [ ! -f "$TEMPLATE_FILE" ]; then
    echo "❌ Template tidak ditemukan di $TEMPLATE_FILE"
    exit 1
fi

# Buat file baru dari template dan ganti placeholder dasar
sed "s/{{ DATE }}/$DATE/g; s/{{ TIME }}/$TIME/g" "$TEMPLATE_FILE" > "$TARGET_FILE"

# Berikan info ke user
echo "✅ Berhasil membuat daily note dari template: $TARGET_FILE"

# Sinkronisasikan langsung ke database agar ID muncul
vault sync-pull
echo "🔄 Database tersinkronisasi."
