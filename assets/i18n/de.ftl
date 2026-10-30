# SPDX-License-Identifier: CC-BY-SA-4.0
# German user interface texts of Grand-Coach (D-32).

## Menu and settings

menu-game = Spiel
menu-new-series = Neue Serie …
menu-end-series = Serie beenden
menu-settings = Einstellungen
setting-language = Sprache
language-de = Deutsch
language-en = English
setting-deck = Kartenbild
deck-french4 = Französisch, vierfarbig (Turnierbild)
deck-french2 = Französisch, zweifarbig
deck-german = Deutsches Blatt (Altenburger Bild)
setting-developer = Entwickleroptionen
setting-show-seed = Seed anzeigen

## New series

new-series-title = Neue Serie
series-fixed = Serie mit fester Spielzahl
series-games = Spiele
series-free = Freies Spiel
option-kontra-re = Mit Kontra und Re
series-seed = Startwert (Seed):
seed-description = Der Seed ist der Startwert des Zufallsgenerators: Derselbe Seed teilt dieselben Karten aus und lässt die Gegner gleich entscheiden, wenn du gleich spielst; so lässt sich eine Serie wiederholen. Erlaubt sind ganze Zahlen von 0 bis 18446744073709551615.
seed-invalid = Kein gültiger Seed: Gib eine ganze Zahl von 0 bis 18446744073709551615 ein.
button-start = Starten
button-cancel = Abbrechen

## Opponents

mode-levels = Gegner einzeln wählen
level-rookie = Grünschnabel
level-rookie-description = Grünschnabel: kennt die Regeln nur teilweise und vertut sich gelegentlich beim Reizen, bei der Spielwahl, beim Drücken und beim Ausspielen.
mode-goldfish = Goldfisch
mode-goldfish-description = Goldfisch: Ein zufällig bestimmter Gegner spielt stark, vergisst aber, welche Karten schon gefallen sind; der andere spielt auf Mittel.
mode-mephisto = Mephisto
opponent-left = Gegner links:
opponent-right = Gegner rechts:
level-easy = Leicht
level-medium = Mittel
level-strong = Stark
level-easy-description = Leicht: spielt nach einfachen Faustregeln und rechnet nichts voraus.
level-medium-description = Mittel: rechnet beim Kartenspiel und bei Kontra/Re einige mögliche Kartenverteilungen durch; reizt, drückt und sagt an nach Faustregeln.
level-strong-description = Stark: wie Mittel, rechnet aber viermal so viele mögliche Kartenverteilungen durch. Die stärkste Stufe; überlegt etwas länger.

## Status line

status-game = Spiel { $number } von { $total }
status-game-free = Spiel { $number }
status-dealer = Geber: { $player }
status-turn = Am Zug: { $player }
status-bid = Reizwert: { $value }
status-declarer = Alleinspieler: { $player }
status-seed = Seed { $seed }
status-levels = Gegner: { $left } (links), { $right } (rechts)
status-opponents = Gegner: { $mode }
status-waiting = { $player } überlegt …

## Players and seats

player-human = Du
player-left = Links
player-right = Rechts
seat-vorhand = Vorhand
seat-mittelhand = Mittelhand
seat-hinterhand = Hinterhand
label-declarer = Alleinspieler
label-cards-tricks = { $cards } Karten, { $tricks } Stiche
label-open-cards = Offene Karten:
label-skat = Skat
label-own-skat = Gedrückt:

## Phases

phase-reizen = Reizen
phase-skataufnahme = Hand oder Skat aufnehmen
phase-druecken = Drücken
phase-ansage = Spielansage
phase-spiel = Kartenspiel
phase-beendet = Spiel beendet
phase-eingepasst = Eingepasst

## Cards and games

suit-kreuz = Kreuz
suit-pik = Pik
suit-herz = Herz
suit-karo = Karo
rank-sieben = Sieben
rank-acht = Acht
rank-neun = Neun
rank-zehn = Zehn
rank-bube = Bube
rank-dame = Dame
rank-koenig = König
rank-ass = Ass
card-name = { $suit }-{ $rank }
game-grand = Grand
game-null = Null
contract-hand = Hand
contract-schneider = Schneider angesagt
contract-schwarz = Schwarz angesagt
contract-ouvert = Ouvert
kontra-level-kontra = Kontra
kontra-level-re = Kontra und Re

## Bidding and actions

call-bid = { $value }
call-hold = Ja ({ $value })
call-pass = Passe
button-bid = { $value } sagen
label-jump-bid = Sprung auf:
button-jump = Springen
button-hold = Ja, { $value } halten
button-pass = Passe
button-hand = Hand spielen
button-take-skat = Skat aufnehmen
hint-discard = Wähle zwei Karten zum Drücken.
button-discard = Drücken
label-announce = Spiel ansagen:
button-announce = Ansagen
button-kontra = Kontra
button-kein-kontra = Kein Kontra
button-re = Re
button-kein-re = Kein Re
hint-play = Du bist dran: Spiel eine Karte.
button-next-game = Nächstes Spiel

## Tricks

trick-current = Aktueller Stich
trick-last = Letzter Stich
trick-winner = { $player } sticht ({ $augen } Augen)

## Results

result-title = Ergebnis
result-won = { $player } gewinnt { $game }.
result-lost = { $player } verliert { $game }.
result-augen = Augen des Alleinspielers: { $augen }
result-tricks = Stiche des Alleinspielers: { $count }
result-spitzen-mit = Mit { $count }
result-spitzen-ohne = Ohne { $count }
result-schneider = Schneider
result-schwarz = Schwarz
result-value = Spielwert: { $value }
result-ueberreizt = Überreizt: Reizwert { $bid }, gewertet mit { $value }
result-score = Listenpunkte: { $score }
result-points = { $player }: { $points } Punkte (Seeger-Fabian)
result-eingepasst = Alle haben gepasst – das Spiel ist eingepasst.

## Score sheet and series

sheet-title = Spielliste
sheet-number = Nr.
sheet-game = Spiel
sheet-total = Summe
sheet-eingepasst = eingepasst
series-finished = Serie beendet nach { $games } Spielen
series-rank = { $rank }. { $player }: { $total } Punkte ({ $won } gewonnen, { $lost } verloren)

## Errors

error-not-your-turn = Du bist nicht am Zug.
error-no-opponent-to-move = Kein Gegner ist am Zug.
error-stale-opponent-move = Der Gegnerzug passt nicht mehr zur aktuellen Lage.
error-game-in-progress = Das laufende Spiel ist noch nicht beendet.
error-series-finished = Die Serie ist beendet. Starte eine neue Serie.
error-not-free-play = Nur eine Serie im freien Spiel kann vorzeitig beendet werden.
error-game-over = Das Spiel ist bereits beendet.
error-unexpected-action = Das ist in der Phase „{ $phase }“ nicht möglich.
error-invalid-bid-value = { $value } ist kein gültiger Reizwert.
error-bid-too-low = { $value } liegt nicht über dem Reizwert { $current }.
error-invalid-discard = Drücke genau zwei Karten aus deiner Hand.
error-invalid-announcement = Diese Kombination aus Spiel und Ansagen ist nicht erlaubt.
error-null-value-below-bid = Dieses Nullspiel ist nur { $value } wert, gereizt wurde { $bid }.
error-card-not-in-hand = { $card } ist nicht auf deiner Hand.
error-must-follow-suit = { $card } geht nicht: Du musst bedienen.
error-kontra-not-allowed = Kontra ist hier nicht erlaubt.

## Coach

menu-coach = Coach
setting-coach-level = Coach meldet sich
coach-level-off = Aus
coach-level-on-request = Auf Nachfrage
coach-level-important = Bei wichtigen Entscheidungen
coach-level-every-move = Bei jedem Zug
setting-coach-warnings = Vor schwachen Zügen warnen
setting-coach-warnings-description = Hält einen deutlich schwächeren Zug an, erklärt warum und lässt dich ihn trotzdem spielen oder neu entscheiden.
setting-coach-figures = Zahlen anzeigen
coach-title = Coach
button-coach-help = Was soll ich bedenken?
coach-status-analysing = Der Coach rechnet …
coach-status-computing = Der Coach rechnet noch an deiner Frage …
coach-status-idle = Frag den Coach, wenn du Hilfe möchtest.
coach-status-waiting = Der Coach meldet sich bei deiner nächsten Entscheidung.
coach-unavailable-cancelled = Die Analyse wurde abgebrochen. Frag erneut.
coach-unavailable-failed = Für diese Entscheidung ist keine Analyse möglich.
coach-checking = Der Coach prüft deinen Zug: { $action }
button-coach-proceed = Trotzdem spielen
button-coach-skip-check = Ohne Prüfung spielen
button-coach-reconsider = Zurück zur Entscheidung
coach-warning-title = Bist du sicher?
coach-warning-text = { $action } ist voraussichtlich deutlich schwächer als { $best }.
coach-warning-loss = Erwarteter Verlust gegenüber dem besten Zug: etwa { $points } Punkte (Seeger-Fabian).
coach-action-bid = { $value } sagen
coach-action-hold = { $value } halten
coach-action-discard = { $cards } drücken
coach-action-announce = { $game } ansagen
coach-decision-bid = Reizen: sagen oder passen
coach-decision-answer = Reizen: halten oder passen
coach-decision-pickup = Hand spielen oder Skat aufnehmen
coach-decision-discard = Drücken
coach-decision-announce = Spiel ansagen
coach-decision-card = Karte spielen
coach-decision-kontra = Kontra oder nicht
coach-decision-re = Re oder nicht
coach-role-bidder = noch ohne Rolle
coach-role-declarer = du bist Alleinspieler
coach-role-defender = du bist Gegenspieler
coach-objective-undecided = Noch kein Spiel angesagt: Jede Option wird danach bewertet, wie das folgende Spiel voraussichtlich ausgeht.
coach-objective-augen = Ziel: mindestens { $target } Augen.
coach-objective-augen-defender = Ziel: den Alleinspieler unter { $target } Augen halten.
coach-objective-all-tricks = Ziel: alle Stiche (Schwarz angesagt).
coach-objective-all-tricks-defender = Ziel: mindestens einen Stich machen (Schwarz angesagt).
coach-objective-no-trick = Ziel (Null): keinen einzigen Stich machen.
coach-objective-no-trick-defender = Ziel (Null): den Alleinspieler zu einem Stich zwingen.
coach-important = Wichtige Entscheidung: Eine naheliegende Option kostet deutlich Punkte.
coach-provisional = Vorläufig: Das Rechenbudget reichte nicht für alle Verteilungen – keine Empfehlung.
coach-section-facts = Was du sicher weißt
coach-section-inferred = Was wahrscheinlich ist (geschätzt, nicht sicher)
coach-section-options = Optionen im Vergleich
coach-section-basis = Worauf die Schätzung beruht
coach-fact-void = { $player } hat keine { $class } mehr (nicht bedient).
coach-fact-trumps-out = Noch { $count } Trümpfe bei den anderen oder im Skat.
coach-fact-augen-won = Augen in Stichen: Alleinspieler { $declarer }, Gegenspieler { $defenders }.
coach-fact-augen-out = Noch { $augen } Augen bei den anderen oder im Skat.
coach-fact-skat-known = Im Skat liegen: { $cards }.
coach-fact-bid-shown = { $player } hat bis { $value } gereizt.
coach-fact-kontra-eligible = { $player } darf noch Kontra sagen.
coach-holder-skat = Der Skat
coach-class-trumps = Trümpfe
coach-class-jacks = Buben
coach-inferred-certain = { $holder } hat sicher { $count } { $class } (aus den Regeln ableitbar).
coach-inferred-hardly = { $holder } hat vermutlich kaum { $class }.
coach-inferred-about-one = { $holder } hat vermutlich etwa einen der { $class }.
coach-inferred-several = { $holder } hat vermutlich mehrere { $class }.
coach-inferred-figures = (im Mittel { $expected }, zwischen { $min } und { $max })
coach-verdict-forced = Nur ein Zug ist möglich.
coach-verdict-clear = { $best } ist klar am besten.
coach-verdict-close = { $best } ist am besten, { $count } weitere sind etwa gleich gut.
coach-verdict-unknown = Keine Bewertung verfügbar.
coach-verdict-provisional = Vorläufiger Vergleich ohne Empfehlung.
coach-judgement-best = am besten
coach-judgement-equivalent = etwa gleich gut
coach-judgement-worse = etwas schwächer
coach-judgement-weak = deutlich schwächer
coach-judgement-screened = nur grob geprüft
coach-judgement-not-evaluated = nicht bewertet
coach-party-own = deine Partei
coach-party-declarer = du als Alleinspieler
coach-party-defenders = ihr Gegenspieler
coach-metric-win = Gewinnchance ({ $party }): { $chance }
coach-metric-score = erwartete Listenpunkte (Seeger-Fabian): { $score } ± { $se }
coach-metric-game-points = erwartete Spielpunkte des Alleinspielers: { $points }
coach-reason-win-more = gewinnt öfter als { $other }
coach-reason-win-less = gewinnt seltener als { $other }
coach-reason-points-more = bringt im Schnitt mehr Punkte als { $other }
coach-reason-points-less = bringt im Schnitt weniger Punkte als { $other }
coach-reason-declarer-more = mehr Spielpunkte für den Alleinspieler als { $other }
coach-reason-declarer-less = weniger Spielpunkte für den Alleinspieler als { $other }
coach-reason-thrown-in-more = wird eher eingepasst als { $other }
coach-reason-thrown-in-less = wird seltener eingepasst als { $other }
coach-reason-same = führt zum selben Ergebnis wie { $other }
coach-reason-uncertain = Der Unterschied liegt innerhalb der Unsicherheit der Schätzung.
coach-more-options = … und { $count } weitere Optionen
coach-basis-exact = Alle möglichen Verteilungen der verdeckten Karten wurden durchgerechnet (exakt im Modell).
coach-basis-sampled = Schätzung über zufällig gezogene Verteilungen der verdeckten Karten.
coach-basis-sampled-figures = Schätzung über { $finished } zufällig gezogene Verteilungen der verdeckten Karten.
coach-assumption-uniform = Annahme: Die verdeckten Karten sind gleich wahrscheinlich verteilt, soweit die Regeln es erlauben.
coach-assumption-bidding = Annahme: Die Gegner reizen nach Faustregeln; Verteilungen, die nicht zum Reizen passen, zählen weniger.
coach-assumption-double-dummy = Annahme: Im Kartenspiel spielen alle so, als sähen sie alle Karten.
coach-assumption-policy = Annahme: Spätere Entscheidungen vor dem Kartenspiel folgen den Faustregeln.
coach-assumption-bid-limit = Annahme: Ein Reizwert heißt „bis hierhin mitgehen“.
error-no-decision = Du hast gerade keine Entscheidung zu treffen.
error-coach-off = Der Coach ist ausgeschaltet.
error-nothing-pending = Es wartet kein angehaltener Zug.
error-action-pending = Entscheide zuerst über den angehaltenen Zug.
error-stale-coach-result = Die Analyse passt nicht mehr zur aktuellen Lage.
