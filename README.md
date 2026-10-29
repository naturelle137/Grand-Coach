# Grand Coach

[Deutsch](#deutsch) · [English](#english)

## Deutsch

### Über das Projekt

Skat-Trainer für PC (Linux und Windows 11): vollständiges Skat nach
Internationaler Skatordnung gegen zwei Computergegner. Dazu kommt ein
abschaltbarer Live-Coach, der den **Denkweg** erklärt, nicht nur den besten
Zug. Schwerpunkte sind Kartenzählen und das Hineinversetzen in die Gegner.

### Funktionen (geplant)

- Skat nach ISkO, optional mit Kontra/Re, Wertung nach Seeger-Fabian
- Gegnerstärke in mehreren Stufen
- Live-Coach in Stufen: aus / auf Anfrage / wichtige Entscheidungen / jeder Zug
- Kartenzählhilfe und Quiz
- Offener Modus, Replay, Analyse nach dem Spiel
- Umschaltbare Kartenbilder (französisch vier- und zweifarbig, deutsches
  Blatt); Oberfläche auf Deutsch und Englisch

Details: [docs/vision.md](docs/vision.md).

### Status

Erste spielbare Version (Meilenstein M2): eine Serie gegen zwei einfache
Computergegner, mit Kontra/Re, drei Kartenbildern und deutscher oder
englischer Oberfläche. Unter Linux getestet, Windows 11 noch nicht.
Fortschritt: [docs/roadmap.md](docs/roadmap.md).

### Systemvoraussetzungen

Linux (Wayland oder X11) mit OpenGL bzw. EGL; Windows 11 ist das zweite
Zielsystem, aber noch nicht getestet.

### Installation

*Noch offen. Es gibt noch keine Releases.*

### Aus dem Quellcode bauen

Benötigt Rust (stable, ab 1.95). Im Wurzelverzeichnis:

```sh
cargo build --release
cargo test
```

Weitere Prüfungen stehen in [CONTRIBUTING.md](CONTRIBUTING.md).

### Benutzung

```sh
cargo run --release -p gui-egui
```

Du spielst unten, die Gegner sitzen links und rechts. Über „Spiel → Neue
Serie …“ wählst du feste Spielzahl (z. B. 36) oder freies Spiel, Kontra/Re
und den Startwert; unter „Einstellungen“ Sprache und Kartenbild. Rechts
stehen der letzte Stich und die Spielliste (Seeger-Fabian).

### Mitwirken

Siehe [CONTRIBUTING.md](CONTRIBUTING.md) und den
[Verhaltenskodex](CODE_OF_CONDUCT.md).

### Lizenz

- **Code**: [EUPL-1.2](LICENSE). Quelldateien tragen die Kopfzeile
  `SPDX-License-Identifier: EUPL-1.2`.
- **Grafiken, Dokumentation und sonstige Inhalte** (`assets/`, `docs/`,
  Texte): [CC BY-SA 4.0](LICENSE-ASSETS).
- **Fremde Dateien und Bibliotheken** nur mit kompatibler Lizenz; Herkunft und
  Lizenz stehen in [docs/third-party.md](docs/third-party.md).

## English

### About

A Skat trainer for PC (Linux and Windows 11): full Skat according to the
International Skat Rules against two computer opponents, plus an optional live
coach that explains the **line of thought** rather than just the best move.
Focus: card counting and putting yourself in the opponents' shoes.

### Features (planned)

- Skat per ISkO, optional Kontra/Re, Seeger-Fabian scoring
- Several opponent strength levels
- Live coach with levels: off / on request / important decisions / every move
- Card-counting aid and quiz
- Open mode, replay, post-game analysis
- Switchable card faces (French four- and two-colour, German deck); UI in
  German and English

Details: [docs/vision.md](docs/vision.md).

### Status

First playable version (milestone M2): a series against two simple
computer opponents, with Kontra/Re, three card faces and a German or
English UI. Tested on Linux, not yet on Windows 11.
Progress: [docs/roadmap.md](docs/roadmap.md).

### Requirements

Linux (Wayland or X11) with OpenGL or EGL; Windows 11 is the second target
platform but has not been tested yet.

### Installation

*To be defined. No releases yet.*

### Building from source

Requires Rust (stable, 1.95 or later). From the repository root:

```sh
cargo build --release
cargo test
```

Further checks are listed in [CONTRIBUTING.md](CONTRIBUTING.md).

### Usage

```sh
cargo run --release -p gui-egui
```

You play at the bottom; the opponents sit left and right. "Game → New
series …" chooses a fixed number of games (e.g. 36) or free play,
Kontra/Re and the seed; "Settings" switches language and card faces. The
right-hand side shows the last trick and the score sheet (Seeger-Fabian).

### Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) and the
[Code of Conduct](CODE_OF_CONDUCT.md).

### Licence

- **Code**: [EUPL-1.2](LICENSE). Source files carry the header
  `SPDX-License-Identifier: EUPL-1.2`.
- **Graphics, documentation and other content** (`assets/`, `docs/`, texts):
  [CC BY-SA 4.0](LICENSE-ASSETS).
- **Third-party files and libraries** only with a compatible licence; origin
  and licence are listed in [docs/third-party.md](docs/third-party.md).
