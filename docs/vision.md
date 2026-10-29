# Vision: Grand-Coach

## Ziel

Ein Skat-Trainer für PC (Linux, Windows 11). Man spielt vollständiges Skat
gegen zwei Computergegner und wird dabei von einem abschaltbaren Live-Coach
begleitet. Der Coach **erklärt den Denkweg**, nicht nur den besten Zug:

1. **Worüber musst du jetzt nachdenken?** (z. B. Trumpfverteilung, Blankkarten,
   Stand der Augen, was das Reizen verraten hat)
2. **Welche Optionen hast du?**
3. **Wie bewerten sie sich, und warum solltest du dich für eine entscheiden?**

Das gilt beim Reizen, Skat aufnehmen und Drücken, bei der Spielwahl und beim
Ausspielen und Zugeben.

Der Nutzer will vor allem das **Kartenzählen** und das **Hineinversetzen in
den Gegner** verbessern.

## Features

### Spiel
- Vollständiges Skat nach **Internationaler Skatordnung (ISkO)**: Reizen,
  Hand, Skat, Drücken, Farbspiele, Grand, Null (Hand/Ouvert),
  Schneider/Schwarz angesagt, Ouvert.
- **Kontra/Re** als Option (abschaltbar).
- Passen alle, wird **eingepasst** (neu gegeben). Kein Ramsch, kein Bock.
- Wertung nach **Seeger-Fabian**.
- **Serien** mit fester Spielzahl (z. B. 36) und **freies Spiel**.
- **Gegnerstärke in 3–4 Stufen**. Die Stufen unterscheiden sich in Rechentiefe,
  Gedächtnis (zählt der Gegner Karten?) und Fehlerquote. Gegner schummeln
  nie: Sie sehen nur, was ein Mensch an ihrem Platz sähe.
- **Replay**: Abgeschlossene Spiele Stich für Stich ansehen, mit
  Coach-Kommentar. **Gleiches Blatt neu spielen**, um Alternativen auszuprobieren.
- Normales Spiel ganz ohne Coach.

### Coach
- **Eingriffsstufen**: aus / auf Anfrage (Hilfe-Knopf) / bei wichtigen
  Entscheidungen / bei jedem Zug.
- **Darstellung**: verbale Erklärung plus Kennzahlen je Option
  (Gewinnwahrscheinlichkeit, erwartete Punkte). Die Zahlen sind ausblendbar.
- **Analyse nach dem Spiel**: die 2–3 entscheidenden Momente, jeweils mit der
  besseren Alternative und Begründung.
- **Statistik über Zeit**: Fehlertypen sammeln (z. B. „Trumpfstand falsch
  eingeschätzt“), damit Schwächen sichtbar werden.

### Kartenzählen
- **Zählhilfe** (zuschaltbar): gespielte Karten, offene Trümpfe, Augen beider
  Parteien, erkannte Fehlfarben der Gegner.
- **Quiz im Spiel**: Das Spiel fragt zwischendurch z. B. „Wie viele Trümpfe
  sind noch draußen?“ und vergleicht die Antwort mit der Wahrheit.
- **Trainingsmodus** (spätere Ausbaustufe): gezielte Übungssituationen
  außerhalb eines vollen Spiels.

### Offener Modus
- Der Spieler sieht alle Karten. Der Coach erklärt weiterhin nur das **eigene**
  Spiel und zeigt, **wie man die gegnerischen Blätter im verdeckten Spiel hätte
  erschließen können**: Welche Information lieferten Reizen, Abwerfen und
  Bedienen?

### Darstellung
- Schlichte Grafik im Stil der 90er/2000er. Gut erkennbare Karten haben
  Vorrang vor Effekten.
- **Kartenbild umschaltbar**. Standard ist das französische Turnierbild mit
  vier Farben (Kreuz schwarz, Pik grün, Herz rot, Karo gelb). Dazu kommt das
  klassische zweifarbige Bild, später evtl. ein deutsches Blatt.
- Karten werden **selbst erzeugt** (per Skript generierte SVGs).
- Oberfläche und Coach-Texte auf **Deutsch und Englisch**.

## Architektur (Leitlinie)

```
skat-core      Regel-Engine: rein, deterministisch (Seed), ohne GUI/IO
   ↓
skat-analysis  Kartenzähler/Wissensstand, Gegnermodell (Verteilungen aus
               Reizen/Abwürfen), Bewertung der Optionen (Monte-Carlo +
               Double-Dummy-Löser)
   ↓
skat-ai        Gegner-KI (Stärkestufen)  │  skat-coach  strukturierte
                                         │  Erklärungen (Überlegungen,
                                         │  Optionen, Bewertung)
   ↓
skat-app       Anwendungsschnittstelle (Fassade): Sitzung, Befehle, Ereignisse,
               Sichtmodelle. Einzige API, die Frontends nutzen.
   ↓
gui-egui       erstes Frontend (Rust + egui)
gui-godot      späteres Frontend (Godot über godot-rust/gdext)
```

Grundsätze:
- **Der Coach verbalisiert nur, was die Analyse berechnet hat.** Er liefert
  strukturierte Daten (Textschlüssel + Parameter). Übersetzt wird erst an der
  Oberfläche.
- **Informationsmengen sind sauber getrennt**: KI und Coach arbeiten auf der
  Sicht eines Spielers, nie auf dem vollständigen Spielzustand. Nur der offene
  Modus und die Nachanalyse dürfen alles sehen, und das ist ausdrücklich
  gekennzeichnet.
- **Frontend-unabhängig**: `skat-app` kennt weder egui noch Godot. Befehle,
  Ereignisse und Sichtmodelle sind einfache, serialisierbare Datentypen. So
  lässt sich Godot später ohne Eingriff in Engine oder Coach anbinden.

## Abgrenzung (nicht im Ziel)

- Kein Online- oder Mehrspieler-Modus, keine menschlichen Mitspieler.
- Kein Ramsch, kein Bock, keine weiteren Kneipenregeln (evtl. später als Option).
- Keine Mobil- oder Web-Version (Godot könnte das später ermöglichen, ist aber
  kein Ziel).
- Keine Echtgeld- oder Turnierfunktionen.
- Keine aufwändige Grafik oder Animation.

## Getroffene Entscheidungen

Siehe [decisions.md](decisions.md).
