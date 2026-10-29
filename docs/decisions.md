# Entscheidungslog

Format: Nummer, Datum, Entscheidung, kurze Begründung. Spätere Änderungen als
neuer Eintrag, der den alten ersetzt („ersetzt D-xx“).

| Nr | Datum | Entscheidung | Begründung |
|----|-------|--------------|------------|
| D-01 | 2026-10-29 | Lizenz Code: EUPL-1.2; Inhalte (assets/, docs/): CC BY-SA 4.0. SPDX-Kopfzeile in jeder Quelldatei. Die frühere MIT-LICENSE aus dem Initial Commit ist ersetzt. | Vorgabe des Projektinhabers. |
| D-02 | 2026-10-29 | Regelwerk: ISkO strikt, alle ISkO-Spielarten im MVP. | Eine eindeutige Referenz. Eine vollständige Engine ist Voraussetzung für Coach und KI. |
| D-03 | 2026-10-29 | Kontra/Re als abschaltbare Option. | In der ISkO nur bei Vereinbarung üblich. Für das Training trotzdem relevant. |
| D-04 | 2026-10-29 | Kein Ramsch, kein Bock. Passen alle, wird eingepasst. | Entspricht der ISkO und hält den Umfang klein. |
| D-05 | 2026-10-29 | Wertung Seeger-Fabian. Serien mit fester Länge und freies Spiel. | Turnierstandard, vergleichbare Leistung. |
| D-06 | 2026-10-29 | Replay mit Coach-Kommentar und „gleiches Blatt neu spielen“. Partien deterministisch per Seed. | Lernen durch Vergleich. Determinismus hilft auch beim Testen. |
| D-07 | 2026-10-29 | Gegnerstärke in 3–4 Stufen (Rechentiefe, Gedächtnis, Fehlerquote). KI sieht nie fremde Karten. | Faires Training, verständliche Wahl. |
| D-08 | 2026-10-29 | Coach-Eingriff in Stufen: aus / auf Anfrage / wichtige Entscheidungen / jeder Zug. Darstellung als Text plus ausblendbare Kennzahlen. | Der Lernende steuert selbst, wie viel Hilfe er bekommt. |
| D-09 | 2026-10-29 | Feedback: Analyse nach dem Spiel und Fehlerstatistik über Zeit. Keine Warnung vor dem Zug. | Selbst denken lassen, danach auswerten. |
| D-10 | 2026-10-29 | Kartenzähl-Quiz im Spiel. Eigener Trainingsmodus als spätere Ausbaustufe. | Kartenzählen ist die größte Schwäche des Nutzers. |
| D-11 | 2026-10-29 | Tech-Stack: Rust (Cargo-Workspace) + egui. Langfristig Godot über godot-rust (gdext) als zweites Frontend. | Schnell genug für Monte-Carlo auf allen Kernen, eine Binary pro Plattform. egui passt zum schlichten Stil. |
| D-12 | 2026-10-29 | Frontends sprechen nur mit `skat-app` (Befehle, Ereignisse, Sichtmodelle als einfache Datentypen). Kein Frontend-Code in Engine, Analyse, KI oder Coach. | Godot soll ohne Eingriffe in die Kernlogik anbindbar sein. |
| D-13 | 2026-10-29 | Coach erzeugt strukturierte Erklärungen (Textschlüssel + Parameter). Übersetzung erst in der Präsentation. | „Der Coach verbalisiert nur, was berechnet wurde.“ Funktioniert so mit beiden Frontends und beiden Sprachen. |
| D-14 | 2026-10-29 | Kartenbild umschaltbar: Standard ist das französische Turnierbild mit vier Farben, dazu das klassische zweifarbige Bild. Karten selbst per Skript als SVG erzeugt. | Gute Erkennbarkeit, keine Lizenzrisiken. |
| D-15 | 2026-10-29 | Oberfläche zweisprachig (Deutsch/Englisch). Code, Bezeichner und Kommentare auf Englisch. Skat-Fachbegriffe bleiben deutsch (Grand, Null, Skat, Schneider …). | Zugänglichkeit. Fachbegriffe sind international üblich. |

## Offene Punkte

- Detailregeln Kontra/Re (wer darf wann, Wertung) bei der Umsetzung gegen die
  aktuelle ISkO und die DSkV-Ligaregeln prüfen.
- i18n-Bibliothek (Kandidat: Fluent / `fluent-rs`) und SVG-Rendering
  (Kandidat: `resvg`, oder vorab gerasterte PNGs) bei Projektstart festlegen.
- Zusammenhang „Stärkestufe ↔ Parameter“ empirisch kalibrieren (KI gegen KI).
