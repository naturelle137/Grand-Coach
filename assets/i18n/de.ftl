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

## New series

new-series-title = Neue Serie
series-fixed = Serie mit fester Spielzahl
series-games = Spiele
series-free = Freies Spiel
option-kontra-re = Mit Kontra und Re
series-seed = Startwert (Seed):
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
