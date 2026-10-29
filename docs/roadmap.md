# Roadmap

**Current status:** M3 done (shared analysis, issue #7,
[analysis.md](analysis.md); accepted per D-39). M4 (issue #9) is
implemented and calibrated ([ai.md](ai.md), D-40); the interactive smoke
test of the strength levels is still open, so M4 is not marked done. The
interactive smoke test on Windows 11 is still open (D-36,
[testing.md](testing.md#platform-verification-m2)).

The milestones build on each other (see the architecture in
[vision.md](vision.md)). A milestone is done when its "Done when …" holds.
No individual tasks here.

- [x] **M0 – Project foundation**
  Done when vision, decision log, CLAUDE.md, licences and contributor docs are
  in the repository.

- [x] **M1 – Rule engine (`skat-core`)**
  Done when the engine fully covers all ISkO games (bidding, Hand, Skat,
  discarding, suit, Grand, Null and Ouvert, Schneider/Schwarz, game value
  including overbidding, Seeger-Fabian) plus Kontra/Re per D-03, and thousands
  of random games (seeded) run through without rule violations.

- [x] **M2 – First playable version**
  Done when a series against two simple opponents can be played on Linux and
  Windows. Includes the application interface (`skat-app`), the egui
  frontend, self-made cards with switchable card faces, and a German/English
  UI.

- [x] **M3 – Analysis layer (`skat-analysis`)**
  Done when card counter, opponent model (distributions from bidding,
  following suit and discards) and option evaluation (Monte Carlo and
  double-dummy) deliver winning chance and expected points for every
  decision, in playable time, backed by tests against known positions.

- [ ] **M4 – Opponent AI with strength levels**
  Done when 3–4 levels play measurably differently strong (AI-vs-AI series)
  and no level sees other players' cards.

- [ ] **M5 – Live coach**
  Done when the coach shows the line of thought (considerations, options,
  evaluation with reasoning) for bidding, discarding, game choice and play,
  supports the levels off / on request / important / every move plus the
  optional pre-move warning, works in German and English, and every
  statement comes from the analysis layer.

- [ ] **M6 – Card counting**
  Done when the optional counting aid (cards played, trumps out, card points,
  known voids) and the in-game quiz work.

- [ ] **M7 – Open mode, replay, post-game analysis**
  Done when open mode can be played and the coach shows how the opponents'
  hands could have been deduced. Also: games can be replayed trick by trick
  and replayed with the same deal, and after each game the 2–3 decisive
  moments appear.

- [ ] **M8 – Training and release 1.0**
  Done when error statistics over time and the training mode work and there
  are installable packages for Linux and Windows 11.

- [ ] **M9 – Godot frontend** (long term)
  Done when a Godot frontend (gdext) offers the same functionality via
  `skat-app`, without changes to engine, analysis, AI or coach.
