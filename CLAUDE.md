# CLAUDE.md – Grand-Coach

Skat-Trainer für PC (Linux, Windows 11): Skat nach ISkO gegen zwei KI-Gegner,
dazu ein abschaltbarer Coach, der den Denkweg erklärt (Worüber nachdenken?
Welche Optionen? Wie bewerten sie sich?). Schwerpunkte: Kartenzählen und
Hineinversetzen in den Gegner.

## Gedächtnis – zuerst lesen
- `docs/vision.md`: Ziele, Features, Abgrenzung, Architektur
- `docs/decisions.md`: Entscheidungslog (D-xx) und offene Punkte
- `docs/roadmap.md`: Meilensteine und aktueller Stand (nach jedem Meilenstein aktualisieren)
- `docs/third-party.md`: Herkunft und Lizenz fremder Dateien

Neue Grundsatzentscheidungen kommen als neuer D-Eintrag in `docs/decisions.md`.
Die Dokumente bleiben knapp.

## Tech-Stack
Rust (Cargo-Workspace), GUI mit egui. Später zusätzlich Godot über
godot-rust/gdext (D-11).

## Architektur (geplante Crates)
```
crates/skat-core      Regel-Engine: rein, deterministisch (Seed), kein IO, keine GUI
crates/skat-analysis  Kartenzähler, Gegnermodell, Optionsbewertung (Monte-Carlo, Double-Dummy)
crates/skat-ai        Gegner-KI mit Stärkestufen
crates/skat-coach     strukturierte Erklärungen (Textschlüssel + Parameter), keine Texte
crates/skat-app       Anwendungsschnittstelle: Sitzung, Befehle, Ereignisse, Sichtmodelle
crates/gui-egui       Frontend egui
tools/cardgen         erzeugt Karten-SVGs nach assets/cards/
```
Abhängigkeiten nur von oben nach unten: core ← analysis ← ai/coach ← app ← gui.

## Harte Regeln
- **Coach und KI rechnen nicht getrennt.** Beide nutzen `skat-analysis`. Der
  Coach verbalisiert nur, was dort berechnet wurde.
- **Informationsmengen trennen.** KI und Coach erhalten nur die Sicht eines
  Spielers, nie den vollständigen Zustand. Ausnahmen (offener Modus,
  Nachanalyse) sind im Typsystem sichtbar.
- **Frontends nutzen ausschließlich `skat-app`.** Keine egui- oder
  Godot-Typen unterhalb von `gui-*`. Die API-Typen sind einfache Datentypen
  (serde-fähig), damit Godot später ohne Kernänderungen andockt.
- **Kein Text in der Logik.** Nutzertexte (Deutsch/Englisch) liegen in
  Übersetzungsdateien unter `assets/i18n/`.
- **Determinismus.** Jeder Zufall läuft über einen übergebenen Seed/RNG,
  damit Tests, Replays und „gleiches Blatt neu“ funktionieren.
- Regelfragen entscheidet die ISkO. Kein Ramsch, kein Bock. Kontra/Re ist
  optional.

## Konventionen
- Code, Bezeichner und Kommentare auf Englisch. Skat-Fachbegriffe bleiben
  deutsch (`Grand`, `Null`, `Skat`, `Schneider`, `Schwarz`, `Hand`,
  `Ouvert`, `Kontra`, `Re`).
- Jede Quelldatei beginnt mit der SPDX-Kopfzeile:
  ```rust
  // SPDX-License-Identifier: EUPL-1.2
  ```
  Bei Skripten mit `#`-Kommentaren entsprechend
  `# SPDX-License-Identifier: EUPL-1.2`.
- Erzeugte oder gezeichnete Inhalte unter `assets/` und `docs/` stehen unter
  CC BY-SA 4.0 (`LICENSE-ASSETS`). SVG-Dateien bekommen bei Bedarf einen
  Kommentar `SPDX-License-Identifier: CC-BY-SA-4.0`.
- Fremde Grafiken oder Bibliotheken nur mit kompatibler Lizenz. Jede fremde
  Datei vor dem Einchecken in `docs/third-party.md` eintragen. Keine NC- oder
  ND-Lizenzen, keine Dateien mit unklarer Herkunft.
- Regel-Engine und Analyse sind testgetrieben. Regelfälle bekommen
  Unit-Tests mit festen Verteilungen.

## Befehle (sobald Code existiert)
```
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all
```
