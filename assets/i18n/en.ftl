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
setting-developer = Developer options
setting-show-seed = Show seed

## New series

new-series-title = New series
series-fixed = Series with a fixed number of games
series-games = games
series-free = Free play
option-kontra-re = With Kontra and Re
series-seed = Seed:
seed-description = The seed is the start value of the random generator: the same seed deals the same cards and, when you play the same way, makes the opponents decide the same way, so a series can be replayed. Allowed are whole numbers from 0 to 18446744073709551615.
seed-invalid = Not a valid seed: enter a whole number from 0 to 18446744073709551615.
button-start = Start
button-cancel = Cancel

## Opponents

mode-levels = Choose each opponent
level-rookie = Rookie
level-rookie-description = Rookie: knows the rules only in part and now and then misjudges the bidding, the game choice, the discard or a card.
mode-goldfish = Goldfish
mode-goldfish-description = Goldfish: one opponent, chosen at random, plays strong but forgets which cards have been played; the other plays Medium.
mode-mephisto = Mephisto
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
status-opponents = Opponents: { $mode }
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

## Coach

menu-coach = Coach
setting-coach-level = Coach speaks up
coach-level-off = Off
coach-level-on-request = On request
coach-level-important = At important decisions
coach-level-every-move = At every move
setting-coach-warnings = Warn before weak moves
setting-coach-warnings-description = Holds back a clearly weaker move, explains why and lets you play it anyway or decide again.
setting-coach-figures = Show figures
coach-title = Coach
button-coach-considerations = What should I consider?
button-coach-situation = What is the situation?
button-coach-figures = Show the figures
coach-section-situation = The situation
coach-situation-forced = Only one move is possible.
coach-situation-one = One option is clearly better than all others. Which one?
coach-situation-several = { $count } options are about equally good.
coach-situation-unclear = The situation cannot be judged reliably.
coach-section-figures = Figures per option (not judged – compare them yourself)
coach-note = Your last move ({ $action }): { $note }
coach-note-good = good.
coach-note-about-equal = about as good as the best.
coach-note-somewhat-weaker = somewhat weaker than the best.
coach-note-clearly-weaker = clearly weaker than the best.
coach-note-forced = there was no choice.
coach-note-uncertain = no reliable judgement.
coach-note-computing = the coach is still judging it …
coach-debriefing-title = Debriefing
coach-debriefing-computing = The coach is still evaluating { $count } decisions …
coach-debriefing-none = No particularly good or weak moments in this game.
coach-moment-good = Strong: { $decision }
coach-moment-bad = Weak: { $decision }
coach-moment-trick = trick { $trick }
coach-moment-chosen = You chose { $action }: { $judgement }.
coach-moment-better = { $best } was better:
coach-moment-trap = You avoided { $trap }, which would have been clearly weaker:
coach-moment-trap-loss = Expected loss with it: about { $points } points (Seeger-Fabian).
coach-moment-knew = What you knew at that moment
coach-status-analysing = The coach is thinking …
coach-status-computing = The coach is still working on your question …
coach-status-idle = Ask the coach whenever you want help.
coach-status-waiting = The coach speaks up at your next decision.
coach-unavailable-cancelled = The analysis was cancelled. Ask again.
coach-unavailable-failed = No analysis is possible for this decision.
coach-checking = The coach is checking your move: { $action }
button-coach-proceed = Play it anyway
button-coach-skip-check = Play without check
button-coach-reconsider = Back to the decision
coach-warning-title = Are you sure?
coach-warning-text = { $action } is likely clearly weaker than the best option. Think again.
coach-warning-loss = Expected loss against the best move: about { $points } points (Seeger-Fabian).
coach-action-bid = Bid { $value }
coach-action-hold = Hold { $value }
coach-action-discard = Discard { $cards }
coach-action-announce = Announce { $game }
coach-decision-bid = Bidding: bid or pass
coach-decision-answer = Bidding: hold or pass
coach-decision-pickup = Play Hand or pick up the Skat
coach-decision-discard = Discarding
coach-decision-announce = Announce the game
coach-decision-card = Play a card
coach-decision-kontra = Kontra or not
coach-decision-re = Re or not
coach-role-bidder = no role yet
coach-role-declarer = you are the declarer
coach-role-defender = you are a defender
coach-objective-undecided = No game announced yet: each option is valued by how the following game is likely to end.
coach-objective-augen = Goal: at least { $target } Augen.
coach-objective-augen-defender = Goal: keep the declarer below { $target } Augen.
coach-objective-all-tricks = Goal: every trick (Schwarz announced).
coach-objective-all-tricks-defender = Goal: take at least one trick (Schwarz announced).
coach-objective-no-trick = Goal (Null): take no trick at all.
coach-objective-no-trick-defender = Goal (Null): force the declarer to take a trick.
coach-important = Important decision: an obvious option costs clearly.
coach-provisional = Provisional: the work budget did not cover every distribution – no reliable reading.
coach-section-facts = What you know for sure
coach-section-inferred = What is likely (estimated, not certain)
coach-section-options = Possible moves (not judged)
coach-section-basis = What the estimate rests on
coach-fact-void = { $player } has no { $class } left (did not follow suit).
coach-fact-trumps-out = { $count } trumps are still with the others or in the Skat.
coach-fact-augen-won = Augen in tricks: declarer { $declarer }, defenders { $defenders }.
coach-fact-augen-out = { $augen } Augen are still with the others or in the Skat.
coach-fact-skat-known = The Skat holds: { $cards }.
coach-fact-bid-shown = { $player } bid up to { $value }.
coach-fact-kontra-eligible = { $player } may still say Kontra.
coach-holder-skat = The Skat
coach-class-trumps = trumps
coach-class-jacks = Jacks
coach-inferred-certain = { $holder } certainly holds { $count } { $class } (follows from the rules).
coach-inferred-hardly = { $holder } probably holds hardly any { $class }.
coach-inferred-about-one = { $holder } probably holds about one of the { $class }.
coach-inferred-several = { $holder } probably holds several { $class }.
coach-inferred-figures = (on average { $expected }, between { $min } and { $max })
coach-judgement-best = best
coach-judgement-equivalent = about as good
coach-judgement-worse = somewhat weaker
coach-judgement-weak = clearly weaker
coach-judgement-screened = only roughly checked
coach-judgement-not-evaluated = not evaluated
coach-party-own = your side
coach-party-declarer = you as declarer
coach-party-defenders = you defenders
coach-metric-win = Winning chance ({ $party }): { $chance }
coach-metric-score = expected list points (Seeger-Fabian): { $score } ± { $se }
coach-metric-game-points = expected game points of the declarer: { $points }
coach-reason-win-more = wins more often than { $other }
coach-reason-win-less = wins less often than { $other }
coach-reason-points-more = scores more points on average than { $other }
coach-reason-points-less = scores fewer points on average than { $other }
coach-reason-declarer-more = more game points for the declarer than { $other }
coach-reason-declarer-less = fewer game points for the declarer than { $other }
coach-reason-thrown-in-more = more often thrown in than { $other }
coach-reason-thrown-in-less = less often thrown in than { $other }
coach-reason-same = leads to the same result as { $other }
coach-reason-uncertain = The difference is within the uncertainty of the estimate.
coach-more-options = … and { $count } more options
coach-basis-exact = Every possible distribution of the hidden cards was evaluated (exact within the model).
coach-basis-sampled = Estimate over randomly drawn distributions of the hidden cards.
coach-basis-sampled-figures = Estimate over { $finished } randomly drawn distributions of the hidden cards.
coach-assumption-uniform = Assumption: the hidden cards are equally likely anywhere the rules allow.
coach-assumption-bidding = Assumption: the opponents bid by rules of thumb; distributions that do not fit their bidding count less.
coach-assumption-double-dummy = Assumption: in card play everybody plays as if seeing every card.
coach-assumption-policy = Assumption: later decisions before card play follow the rules of thumb.
coach-assumption-bid-limit = Assumption: a bid value means "go along up to here".
error-no-decision = You have no decision to take right now.
error-coach-off = The coach is switched off.
error-nothing-pending = No move is being held back.
error-action-pending = First decide about the move being held back.
error-stale-coach-result = The analysis no longer fits the current position.
