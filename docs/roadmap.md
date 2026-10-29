# Roadmap

**Aktueller Stand:** M0 abgeschlossen. Als Nächstes kommt M1 (Regel-Engine).

Die Meilensteine bauen aufeinander auf (siehe Architektur in
[vision.md](vision.md)). Ein Meilenstein ist erledigt, wenn sein
„Fertig, wenn …“ erfüllt ist. Einzelaufgaben gehören nicht hierher.

- [x] **M0 – Projektgrundlage**
  Fertig, wenn Vision, Entscheidungslog, CLAUDE.md und Lizenzen im Repo stehen.

- [ ] **M1 – Regel-Engine (`skat-core`)**
  Fertig, wenn die Engine alle ISkO-Spiele vollständig abbildet: Reizen, Hand,
  Skat, Drücken, Farbe, Grand, Null und Ouvert, Schneider/Schwarz,
  Kontra/Re, Spielwert inklusive Überreizen, Seeger-Fabian. Außerdem müssen
  Tausende zufälliger Partien (mit Seed) ohne Regelverstoß durchlaufen.

- [ ] **M2 – Erste spielbare Version**
  Fertig, wenn man unter Linux und Windows eine Serie gegen zwei einfache
  Gegner spielen kann. Das umfasst die Anwendungsschnittstelle (`skat-app`),
  das egui-Frontend, selbst erzeugte Karten mit umschaltbarem Kartenbild und
  die Oberfläche auf Deutsch und Englisch.

- [ ] **M3 – Analyse-Schicht (`skat-analysis`)**
  Fertig, wenn Kartenzähler, Gegnermodell (Verteilungen aus Reizen, Bedienen
  und Abwerfen) und Optionsbewertung (Monte-Carlo und Double-Dummy) für jede
  Entscheidung Gewinnchance und erwartete Punkte liefern. Das muss in
  spielbarer Zeit gehen und durch Tests gegen bekannte Stellungen abgesichert
  sein.

- [ ] **M4 – Gegner-KI mit Stärkestufen**
  Fertig, wenn es 3–4 Stufen gibt, die messbar unterschiedlich stark spielen
  (KI-gegen-KI-Serien). Keine Stufe darf fremde Karten sehen.

- [ ] **M5 – Live-Coach**
  Fertig, wenn der Coach beim Reizen, Drücken, bei der Spielwahl und beim
  Ausspielen den Denkweg zeigt (Überlegungen, Optionen, Bewertung mit
  Begründung). Er muss die Eingriffsstufen aus / auf Anfrage / wichtig /
  jeder Zug können, auf Deutsch und Englisch, und alle Aussagen müssen aus
  der Analyse-Schicht stammen.

- [ ] **M6 – Kartenzählen**
  Fertig, wenn die zuschaltbare Zählhilfe (gespielte Karten, offene Trümpfe,
  Augen, erkannte Fehlfarben) und das Quiz im Spiel funktionieren.

- [ ] **M7 – Offener Modus, Replay, Nachanalyse**
  Fertig, wenn man im offenen Modus spielen kann und der Coach dabei zeigt,
  wie die gegnerischen Blätter erschließbar gewesen wären. Außerdem: Partien
  lassen sich Stich für Stich nachspielen, mit gleichem Blatt neu spielen,
  und nach jedem Spiel erscheinen die 2–3 entscheidenden Momente.

- [ ] **M8 – Training und Release 1.0**
  Fertig, wenn die Fehlerstatistik über Zeit und der eigene Trainingsmodus
  laufen und es installierbare Pakete für Linux und Windows 11 gibt.

- [ ] **M9 – Godot-Frontend** (langfristig)
  Fertig, wenn ein Godot-Frontend (gdext) denselben Funktionsumfang über
  `skat-app` bietet, ohne dass sich Engine, Analyse, KI oder Coach ändern.
