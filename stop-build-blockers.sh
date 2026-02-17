#!/bin/bash
# Stoppt alle Prozesse, die den Build blockieren könnten

echo "Beende blockierende Prozesse..."

pkill -9 -f "cargo" 2>/dev/null && echo "  - cargo beendet" || true
pkill -9 -f "rustc" 2>/dev/null && echo "  - rustc beendet" || true
pkill -9 -f "rust-analyzer" 2>/dev/null && echo "  - rust-analyzer beendet" || true
pkill -9 -f "tauri" 2>/dev/null && echo "  - tauri beendet" || true
pkill -9 -f "vite" 2>/dev/null && echo "  - vite beendet" || true
pkill -9 -f "node.*5173" 2>/dev/null && echo "  - Vite dev server beendet" || true

sleep 2

echo ""
echo "Leere Cargo-Build-Cache..."
cd "$(dirname "$0")/src-tauri" && cargo clean

echo ""
echo "Fertig. Starte Build in neuem Terminal:"
echo "  cd $(dirname "$0")"
echo "  npx tauri build"
echo ""
echo "Tipp: Cursor/IDE kurz schliessen vor dem Build (rust-analyzer blockiert oft)."
