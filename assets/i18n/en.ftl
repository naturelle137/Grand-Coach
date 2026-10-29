# SPDX-License-Identifier: CC-BY-SA-4.0
# English user interface texts of Grand-Coach (D-32).

## Menu and settings

menu-game = Game
menu-new-series = New series …
menu-end-series = End series
menu-settings = Settings
setting-language = Language
language-de = Deutsch
language-en = English
setting-deck = Card faces
deck-french4 = French, four colours (tournament)
deck-french2 = French, two colours
deck-german = German suits (Altenburg pattern)

## New series

new-series-title = New series
series-fixed = Series with a fixed number of games
series-games = games
series-free = Free play
option-kontra-re = With Kontra and Re
series-seed = Seed:
button-start = Start
button-cancel = Cancel

## Opponents

opponent-left = Left opponent:
opponent-right = Right opponent:
level-easy = Easy
level-medium = Medium
level-strong = Strong
level-easy-description = Easy: plays by simple rules of thumb and does not calculate ahead.
level-medium-description = Medium: works through a few possible card distributions for card play and Kontra/Re; bids, discards and announces by rules of thumb.
level-strong-description = Strong: like Medium, but works through four times as many possible card distributions. The strongest level; thinks a little longer.

## Status line

status-game = Game { $number } of { $total }
status-game-free = Game { $number }
status-dealer = Dealer: { $player }
status-turn = To move: { $player }
status-bid = Bid: { $value }
status-declarer = Declarer: { $player }
status-seed = Seed { $seed }
status-levels = Opponents: { $left } (left), { $right } (right)
status-waiting = { $player } is thinking …

## Players and seats

player-human = You
player-left = Left
player-right = Right
seat-vorhand = Forehand
seat-mittelhand = Middlehand
seat-hinterhand = Rearhand
label-declarer = declarer
label-cards-tricks = { $cards } cards, { $tricks } tricks
label-open-cards = Open cards:
label-skat = Skat
label-own-skat = Discarded:

## Phases

phase-reizen = Bidding
phase-skataufnahme = Hand or pick up the Skat
phase-druecken = Discarding
phase-ansage = Announcing the game
phase-spiel = Card play
phase-beendet = Game over
phase-eingepasst = Thrown in

## Cards and games

suit-kreuz = Clubs
suit-pik = Spades
suit-herz = Hearts
suit-karo = Diamonds
rank-sieben = Seven
rank-acht = Eight
rank-neun = Nine
rank-zehn = Ten
rank-bube = Jack
rank-dame = Queen
rank-koenig = King
rank-ass = Ace
card-name = { $rank } of { $suit }
game-grand = Grand
game-null = Null
contract-hand = Hand
contract-schneider = Schneider announced
contract-schwarz = Schwarz announced
contract-ouvert = Ouvert
kontra-level-kontra = Kontra
kontra-level-re = Kontra and Re

## Bidding and actions

call-bid = { $value }
call-hold = Yes ({ $value })
call-pass = Pass
button-bid = Bid { $value }
label-jump-bid = Jump to:
button-jump = Jump
button-hold = Yes, hold { $value }
button-pass = Pass
button-hand = Play Hand
button-take-skat = Pick up the Skat
hint-discard = Choose two cards to discard.
button-discard = Discard
label-announce = Announce the game:
button-announce = Announce
button-kontra = Kontra
button-kein-kontra = No Kontra
button-re = Re
button-kein-re = No Re
hint-play = Your turn: play a card.
button-next-game = Next game

## Tricks

trick-current = Current trick
trick-last = Last trick
trick-winner = { $player } takes the trick ({ $augen } card points)

## Results

result-title = Result
result-won = { $player } wins { $game }.
result-lost = { $player } loses { $game }.
result-augen = Declarer's card points: { $augen }
result-tricks = Declarer's tricks: { $count }
result-spitzen-mit = With { $count }
result-spitzen-ohne = Without { $count }
result-schneider = Schneider
result-schwarz = Schwarz
result-value = Game value: { $value }
result-ueberreizt = Overbid: bid { $bid }, scored as { $value }
result-score = Score: { $score }
result-points = { $player }: { $points } points (Seeger-Fabian)
result-eingepasst = Everybody passed – the deal is thrown in.

## Score sheet and series

sheet-title = Score sheet
sheet-number = No.
sheet-game = Game
sheet-total = Total
sheet-eingepasst = thrown in
series-finished = Series over after { $games } games
series-rank = { $rank }. { $player }: { $total } points ({ $won } won, { $lost } lost)

## Errors

error-not-your-turn = It is not your turn.
error-no-opponent-to-move = No opponent is to move.
error-stale-opponent-move = The opponent move no longer fits the current position.
error-game-in-progress = The current game is not over yet.
error-series-finished = The series is over. Start a new series.
error-not-free-play = Only a free-play series can be ended early.
error-game-over = The game is already over.
error-unexpected-action = That is not possible during “{ $phase }”.
error-invalid-bid-value = { $value } is not a valid bid.
error-bid-too-low = { $value } is not above the current bid of { $current }.
error-invalid-discard = Discard exactly two cards from your hand.
error-invalid-announcement = This combination of game and announcements is not allowed.
error-null-value-below-bid = This Null game is only worth { $value }, but the bid is { $bid }.
error-card-not-in-hand = { $card } is not in your hand.
error-must-follow-suit = You cannot play { $card }: you must follow suit.
error-kontra-not-allowed = Kontra is not allowed here.
