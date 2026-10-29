// SPDX-License-Identifier: EUPL-1.2

//! Generates the card faces of Grand-Coach as SVG (D-14): a four-colour
//! French tournament deck, a classic two-colour French deck and a German
//! deck in the manner of the Altenburger Bild. Every shape is drawn here;
//! indices are stroked paths, so no font is needed to render them.
//!
//! Every deck also has a Mephisto variant (D-41): the Jacks (Unter) carry
//! the index M and a devil, and the Kreuz (Eichel) cards an inverted cross
//! as suit symbol.
//!
//! Output paths are relative to `assets/cards/`: `<deck>/<suit>-<rank>.svg`,
//! `mephisto/<deck>/<suit>-<rank>.svg` and `back.svg`. Generated files are
//! CC BY-SA 4.0 (`LICENSE-ASSETS`).

use std::fmt::Write;

/// Card decks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Deck {
    /// French faces, four colours (default).
    French4,
    /// French faces, two colours.
    French2,
    /// German faces (Eichel, Grün, Rot, Schellen).
    German,
}

impl Deck {
    /// All decks.
    pub const ALL: [Deck; 3] = [Deck::French4, Deck::French2, Deck::German];

    /// Directory name below `assets/cards/`.
    pub const fn dir(self) -> &'static str {
        match self {
            Deck::French4 => "french4",
            Deck::French2 => "french2",
            Deck::German => "german",
        }
    }
}

/// Suit names used in file names, in `skat-core` order.
pub const SUITS: [&str; 4] = ["kreuz", "pik", "herz", "karo"];
/// Rank names used in file names, in `skat-core` order.
pub const RANKS: [&str; 8] = [
    "sieben", "acht", "neun", "zehn", "bube", "dame", "koenig", "ass",
];

const W: f32 = 200.0;
const H: f32 = 300.0;
const LICENSE: &str = "<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->";

/// All generated files as (path below `assets/cards/`, SVG text).
pub fn generate() -> Vec<(String, String)> {
    let mut files = Vec::new();
    for mephisto in [false, true] {
        let prefix = if mephisto { "mephisto/" } else { "" };
        for deck in Deck::ALL {
            for (s, suit) in SUITS.iter().enumerate() {
                for (r, rank) in RANKS.iter().enumerate() {
                    let path = format!("{prefix}{}/{suit}-{rank}.svg", deck.dir());
                    files.push((path, card(deck, s, r, mephisto)));
                }
            }
        }
    }
    files.push(("back.svg".to_owned(), back()));
    files
}

fn svg(body: &str) -> String {
    format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {W} {H}\" width=\"{W}\" height=\"{H}\">\n\
         {LICENSE}\n{body}</svg>\n"
    )
}

fn outline(fill: &str) -> String {
    format!(
        "<rect x=\"2\" y=\"2\" width=\"{}\" height=\"{}\" rx=\"12\" fill=\"{fill}\" stroke=\"#333\" stroke-width=\"2\"/>\n",
        W - 4.0,
        H - 4.0
    )
}

/// The back, shared by all decks.
fn back() -> String {
    let mut body = outline("#ffffff");
    body.push_str(
        "<defs><pattern id=\"lattice\" width=\"16\" height=\"16\" patternUnits=\"userSpaceOnUse\">\
         <rect width=\"16\" height=\"16\" fill=\"#24489a\"/>\
         <path d=\"M0,8 L8,0 L16,8 L8,16 Z\" fill=\"none\" stroke=\"#9fb4e8\" stroke-width=\"1.5\"/>\
         </pattern></defs>\n",
    );
    body.push_str(
        "<rect x=\"12\" y=\"12\" width=\"176\" height=\"276\" rx=\"6\" fill=\"url(#lattice)\" stroke=\"#24489a\" stroke-width=\"2\"/>\n",
    );
    body.push_str(
        "<rect x=\"60\" y=\"110\" width=\"80\" height=\"80\" rx=\"10\" fill=\"#24489a\" stroke=\"#9fb4e8\" stroke-width=\"3\"/>\n",
    );
    body.push_str(&glyphs("GC", 74.0, 132.0, 0.6, "#ffffff"));
    svg(&body)
}

/// Colours of a suit: fill, outline and index ink.
struct Ink {
    fill: &'static str,
    stroke: &'static str,
    index: &'static str,
}

fn ink(deck: Deck, suit: usize) -> Ink {
    const BLACK: Ink = Ink {
        fill: "#1a1a1a",
        stroke: "#1a1a1a",
        index: "#1a1a1a",
    };
    const RED: Ink = Ink {
        fill: "#d42020",
        stroke: "#a01010",
        index: "#d42020",
    };
    const GREEN: Ink = Ink {
        fill: "#1e8a32",
        stroke: "#145c22",
        index: "#1e8a32",
    };
    const YELLOW: Ink = Ink {
        fill: "#f0a800",
        stroke: "#8a5a00",
        index: "#b07800",
    };
    const BROWN: Ink = Ink {
        fill: "#7a4e1c",
        stroke: "#3e2608",
        index: "#6a400e",
    };
    match (deck, suit) {
        (Deck::French4, 0) | (Deck::French2, 0 | 1) => BLACK,
        (Deck::French4, 1) | (Deck::German, 1) => GREEN,
        (Deck::French4, 3) | (Deck::German, 3) => YELLOW,
        (Deck::German, 0) => BROWN,
        _ => RED,
    }
}

/// Suit symbol drawn in a 100 × 100 box centred on the origin. `flipped`
/// marks a symbol that is drawn rotated by 180°; the Mephisto cross is
/// then drawn upright, so it appears inverted on the card.
fn symbol(deck: Deck, suit: usize, mephisto: bool, flipped: bool) -> String {
    let i = ink(deck, suit);
    let (fill, stroke) = (i.fill, i.stroke);
    if mephisto && suit == 0 {
        // Inverted Latin cross: the crossbar sits below the centre.
        let d = if flipped {
            "M-7,-48 H7 V-26 H30 V-12 H7 V48 H-7 V-12 H-30 V-26 H-7 Z"
        } else {
            "M-7,-48 H7 V12 H30 V26 H7 V48 H-7 V26 H-30 V12 H-7 Z"
        };
        return format!(
            "<path d=\"{d}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"3\" stroke-linejoin=\"round\"/>"
        );
    }
    let path = |d: &str| {
        format!(
            "<path d=\"{d}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"3\" stroke-linejoin=\"round\"/>"
        )
    };
    const HEART: &str =
        "M0,42 C-8,32 -46,8 -46,-16 C-46,-38 -18,-46 0,-24 C18,-46 46,-38 46,-16 C46,8 8,32 0,42 Z";
    match (deck, suit) {
        (Deck::German, 0) => format!(
            "<rect x=\"-4\" y=\"-50\" width=\"8\" height=\"14\" fill=\"{stroke}\"/>\
             <path d=\"M-20,-14 C-24,16 -10,44 0,47 C10,44 24,16 20,-14 Z\" fill=\"#c8923e\" stroke=\"{stroke}\" stroke-width=\"3\"/>\
             <path d=\"M-28,-12 C-28,-42 28,-42 28,-12 Z\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"3\"/>\
             <path d=\"M-18,-30 L-8,-14 M-4,-36 L6,-14 M10,-34 L18,-16\" stroke=\"#c8923e\" stroke-width=\"2.5\" fill=\"none\"/>"
        ),
        (Deck::German, 1) => format!(
            "{}<path d=\"M0,-36 V48 M0,-10 L-16,-22 M0,-10 L16,-22 M0,10 L-20,-2 M0,10 L20,-2\" stroke=\"{stroke}\" stroke-width=\"3\" fill=\"none\"/>",
            path(
                "M0,-48 C26,-34 42,-6 32,18 C24,34 10,40 0,44 C-10,40 -24,34 -32,18 C-42,-6 -26,-34 0,-48 Z"
            )
        ),
        (Deck::German, 3) => format!(
            "<circle cx=\"0\" cy=\"-40\" r=\"7\" fill=\"none\" stroke=\"{stroke}\" stroke-width=\"4\"/>\
             <circle cx=\"0\" cy=\"4\" r=\"38\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"3\"/>\
             <path d=\"M-37,-6 H37\" stroke=\"#c01818\" stroke-width=\"6\"/>\
             <path d=\"M-16,24 H16\" stroke=\"{stroke}\" stroke-width=\"4\"/>\
             <circle cx=\"0\" cy=\"31\" r=\"5\" fill=\"{stroke}\"/>"
        ),
        (_, 0) => format!(
            "<circle cx=\"0\" cy=\"-24\" r=\"19\" fill=\"{fill}\"/><circle cx=\"-22\" cy=\"6\" r=\"19\" fill=\"{fill}\"/>\
             <circle cx=\"22\" cy=\"6\" r=\"19\" fill=\"{fill}\"/><circle cx=\"0\" cy=\"0\" r=\"10\" fill=\"{fill}\"/>\
             <path d=\"M-5,4 C-4,24 -10,38 -20,46 H20 C10,38 4,24 5,4 Z\" fill=\"{fill}\"/>"
        ),
        (_, 1) => path(
            "M0,-46 C10,-30 46,-12 46,10 C46,30 22,36 6,22 C8,34 12,42 20,46 H-20 C-12,42 -8,34 -6,22 C-22,36 -46,30 -46,10 C-46,-12 -10,-30 0,-46 Z",
        ),
        (_, 2) => path(HEART),
        _ => path(
            "M0,-48 C10,-30 22,-14 34,0 C22,14 10,30 0,48 C-10,30 -22,14 -34,0 C-22,-14 -10,-30 0,-48 Z",
        ),
    }
}

fn place(content: &str, x: f32, y: f32, scale: f32, rotate: bool) -> String {
    let r = if rotate { " rotate(180)" } else { "" };
    format!("<g transform=\"translate({x},{y}) scale({scale}){r}\">{content}</g>\n")
}

/// Stroked glyph in a 40 × 60 cell, as (advance, SVG).
fn glyph(c: char, color: &str) -> (f32, String) {
    let stroke = format!(
        "fill=\"none\" stroke=\"{color}\" stroke-width=\"7\" stroke-linecap=\"round\" stroke-linejoin=\"round\""
    );
    let path = |d: &str| format!("<path d=\"{d}\" {stroke}/>");
    let ellipse = |cx: u8, cy: u8, rx: u8, ry: u8| {
        format!("<ellipse cx=\"{cx}\" cy=\"{cy}\" rx=\"{rx}\" ry=\"{ry}\" {stroke}/>")
    };
    match c {
        '7' => (44.0, path("M3,4 H37 L15,57")),
        '8' => (
            44.0,
            format!("{}{}", ellipse(20, 16, 12, 12), ellipse(20, 43, 15, 14)),
        ),
        '9' => (
            44.0,
            format!(
                "{}{}",
                ellipse(20, 20, 15, 16),
                path("M35,20 C35,42 26,56 8,57")
            ),
        ),
        '1' => (24.0, path("M4,14 L14,4 V57")),
        '0' => (44.0, ellipse(20, 30, 15, 27)),
        'B' => (
            44.0,
            path("M7,3 V57 H24 C44,57 44,29 24,29 H7 M7,3 H21 C38,3 38,29 21,29"),
        ),
        'D' => (44.0, path("M7,3 V57 H17 C45,57 45,3 17,3 Z")),
        'K' => (44.0, path("M7,3 V57 M35,3 L7,37 M17,26 L37,57")),
        'A' => (44.0, path("M3,57 L20,3 L37,57 M9,39 H31")),
        'M' => (44.0, path("M5,57 V3 L20,34 L35,3 V57")),
        'U' => (44.0, path("M6,3 V38 C6,62 34,62 34,38 V3")),
        'O' => (44.0, ellipse(20, 30, 16, 27)),
        'G' => (
            44.0,
            path("M36,10 C30,2 6,0 4,30 C6,60 34,60 36,40 V32 H22"),
        ),
        'C' => (44.0, path("M36,10 C30,2 6,0 4,30 C6,60 34,60 36,50")),
        _ => panic!("no glyph for {c}"),
    }
}

/// A string of glyphs with its top-left corner at (x, y).
fn glyphs(text: &str, x: f32, y: f32, scale: f32, color: &str) -> String {
    let mut out = String::new();
    let mut advance = 0.0;
    for c in text.chars() {
        let (width, svg) = glyph(c, color);
        out.push_str(&place(&svg, x + advance * scale, y, scale, false));
        advance += width;
    }
    out
}

fn index_text(deck: Deck, rank: usize, mephisto: bool) -> &'static str {
    match (deck, rank) {
        (_, 4) if mephisto => "M",
        (_, 0) => "7",
        (_, 1) => "8",
        (_, 2) => "9",
        (_, 3) => "10",
        (Deck::German, 4) => "U",
        (Deck::German, 5) => "O",
        (_, 4) => "B",
        (_, 5) => "D",
        (_, 6) => "K",
        _ => "A",
    }
}

/// Pip positions of the number cards, from the top.
fn pips(rank: usize) -> &'static [(f32, f32)] {
    match rank {
        0 => &[
            (66.0, 72.0),
            (134.0, 72.0),
            (100.0, 111.0),
            (66.0, 150.0),
            (134.0, 150.0),
            (66.0, 228.0),
            (134.0, 228.0),
        ],
        1 => &[
            (66.0, 72.0),
            (134.0, 72.0),
            (100.0, 111.0),
            (66.0, 150.0),
            (134.0, 150.0),
            (100.0, 189.0),
            (66.0, 228.0),
            (134.0, 228.0),
        ],
        2 => &[
            (66.0, 72.0),
            (134.0, 72.0),
            (66.0, 124.0),
            (134.0, 124.0),
            (100.0, 150.0),
            (66.0, 176.0),
            (134.0, 176.0),
            (66.0, 228.0),
            (134.0, 228.0),
        ],
        _ => &[
            (66.0, 72.0),
            (134.0, 72.0),
            (100.0, 98.0),
            (66.0, 124.0),
            (134.0, 124.0),
            (66.0, 176.0),
            (134.0, 176.0),
            (100.0, 202.0),
            (66.0, 228.0),
            (134.0, 228.0),
        ],
    }
}

/// The upper half of the Mephisto Jack: a devil with horns and a goatee.
fn devil(deck: Deck, suit: usize, flipped: bool) -> String {
    let i = ink(deck, suit);
    let mut out = format!(
        "<path d=\"M66,150 L76,116 Q100,104 124,116 L134,150 Z\" fill=\"{}\" stroke=\"#333\" stroke-width=\"2\"/>\
         <path d=\"M88,108 L100,128 L112,108\" fill=\"#7a0a0a\" stroke=\"#333\" stroke-width=\"2\"/>\
         <path d=\"M86,84 Q76,66 80,54 Q84,70 94,78 Z\" fill=\"#3a0606\" stroke=\"#222\" stroke-width=\"2\"/>\
         <path d=\"M114,84 Q124,66 120,54 Q116,70 106,78 Z\" fill=\"#3a0606\" stroke=\"#222\" stroke-width=\"2\"/>\
         <path d=\"M84,92 Q84,76 100,76 Q116,76 116,92 Q116,104 106,110 L100,122 L94,110 Q84,104 84,92 Z\" fill=\"#c8102e\" stroke=\"#333\" stroke-width=\"2\"/>\
         <path d=\"M89,88 L97,91 M111,88 L103,91\" stroke=\"#1a1a1a\" stroke-width=\"2.5\" stroke-linecap=\"round\"/>\
         <circle cx=\"94\" cy=\"94\" r=\"2\" fill=\"#f0d020\"/><circle cx=\"106\" cy=\"94\" r=\"2\" fill=\"#f0d020\"/>\
         <path d=\"M93,103 Q100,108 107,103\" fill=\"none\" stroke=\"#1a1a1a\" stroke-width=\"2\" stroke-linecap=\"round\"/>",
        i.fill
    );
    out.push_str(&place(
        &symbol(deck, suit, true, flipped),
        64.0,
        78.0,
        0.2,
        false,
    ));
    out
}

/// One half of a court figure (the upper one; the lower is the same half
/// rotated, drawn with `flipped`).
fn figure(deck: Deck, suit: usize, rank: usize, mephisto: bool, flipped: bool) -> String {
    if mephisto && rank == 4 {
        return devil(deck, suit, flipped);
    }
    let i = ink(deck, suit);
    let mut out = format!(
        "<path d=\"M66,150 L76,116 Q100,104 124,116 L134,150 Z\" fill=\"{}\" stroke=\"#333\" stroke-width=\"2\"/>\
         <path d=\"M90,112 L100,130 L110,112\" fill=\"#ffffff\" stroke=\"#333\" stroke-width=\"2\"/>\
         <circle cx=\"100\" cy=\"94\" r=\"16\" fill=\"#f6d7b0\" stroke=\"#333\" stroke-width=\"2\"/>",
        i.fill
    );
    let headwear = match (deck, rank) {
        // King: crown.
        (_, 6) => "<path d=\"M84,82 L86,62 L93,72 L100,58 L107,72 L114,62 L116,82 Z\" fill=\"#e8b820\" stroke=\"#333\" stroke-width=\"2\"/>".to_owned(),
        // Ober: hat with a feather.
        (Deck::German, 5) => format!(
            "<path d=\"M78,84 Q100,62 122,84 Z\" fill=\"{}\" stroke=\"#333\" stroke-width=\"2\"/>\
             <path d=\"M112,74 Q128,52 136,58\" fill=\"none\" stroke=\"#c01818\" stroke-width=\"4\" stroke-linecap=\"round\"/>",
            i.stroke
        ),
        // Dame: veil.
        (_, 5) => "<path d=\"M82,104 Q76,70 100,72 Q124,70 118,104 Q112,86 100,84 Q88,86 82,104 Z\" fill=\"#6a3a8a\" stroke=\"#333\" stroke-width=\"2\"/>".to_owned(),
        // Bube / Unter: cap.
        _ => format!(
            "<path d=\"M82,86 Q98,64 120,80 L126,86 Z\" fill=\"{}\" stroke=\"#333\" stroke-width=\"2\"/>",
            i.stroke
        ),
    };
    out.push_str(&headwear);
    out.push_str(&place(
        &symbol(deck, suit, mephisto, flipped),
        64.0,
        78.0,
        0.2,
        false,
    ));
    out
}

fn card(deck: Deck, suit: usize, rank: usize, mephisto: bool) -> String {
    let i = ink(deck, suit);
    let mut body = outline("#ffffff");
    let sym = symbol(deck, suit, mephisto, false);
    let flipped = symbol(deck, suit, mephisto, true);
    // Corner indices, top left and rotated bottom right.
    let index = glyphs(index_text(deck, rank, mephisto), 12.0, 14.0, 0.42, i.index);
    body.push_str(&index);
    body.push_str(&place(&sym, 24.0, 56.0, 0.2, false));
    body.push_str(&format!(
        "<g transform=\"rotate(180 {} {})\">\n{index}{}</g>\n",
        W / 2.0,
        H / 2.0,
        place(&flipped, 24.0, 56.0, 0.2, false)
    ));
    match rank {
        0..=3 => {
            for &(x, y) in pips(rank) {
                let rotated = y > 150.0;
                let pip = if rotated { &flipped } else { &sym };
                body.push_str(&place(pip, x, y, 0.34, rotated));
            }
        }
        7 => body.push_str(&place(&sym, 100.0, 150.0, 0.9, false)),
        _ => {
            let tint = match i.fill {
                "#1a1a1a" => "#ececec",
                "#d42020" => "#fbe3e3",
                "#1e8a32" => "#e2f3e5",
                "#f0a800" => "#fdf1d2",
                _ => "#f1e6d8",
            };
            let _ = write!(
                body,
                "<rect x=\"44\" y=\"40\" width=\"112\" height=\"220\" rx=\"6\" fill=\"{tint}\" stroke=\"{}\" stroke-width=\"3\"/>\n\
                 <clipPath id=\"frame\"><rect x=\"44\" y=\"40\" width=\"112\" height=\"220\" rx=\"6\"/></clipPath>\n",
                i.stroke
            );
            let half = figure(deck, suit, rank, mephisto, false);
            let lower = figure(deck, suit, rank, mephisto, true);
            let _ = write!(
                body,
                "<g clip-path=\"url(#frame)\">{half}<g transform=\"rotate(180 100 150)\">{lower}</g></g>\n\
                 <path d=\"M44,150 H156\" stroke=\"{}\" stroke-width=\"2\"/>\n",
                i.stroke
            );
        }
    }
    svg(&body)
}
