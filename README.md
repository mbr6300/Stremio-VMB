# Stremio VMB

Media Browser mit RealDebrid-Integration, lokaler Bibliothek und Musik-Player. Erstellt mit Tauri 2, React, TypeScript und Vite.

## Features

- **Streams / Discover**: TMDb-basierte Empfehlungen, Suche, RealDebrid-Streams
- **Bibliothek**: Lokale Medien scannen, Metadaten von TMDb
- **Musik**: Lokale Musikbibliothek, Alben, Playlists, Radio
- **Details**: Poster, Besetzung, Trivia & Schauspieler-Overlays (Perplexity)
- **Player**: Externe Player (z.B. VLC) oder Standard-Player

## Voraussetzungen

- [Node.js](https://nodejs.org/) (LTS)
- [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

## Entwicklung

```bash
pnpm install
pnpm tauri dev
```

## Build

```bash
pnpm tauri build
```

## Konfiguration

In den Einstellungen können u.a. konfiguriert werden:

- **TMDb API-Key** – für Metadaten, Discover, Trivia
- **Perplexity API-Key** – für Anekdoten, Trivia-Facts, KI-Empfehlungen
- **RealDebrid API-Key** – für Streams
- **Medien-Pfade** – für Bibliothek und Musik
