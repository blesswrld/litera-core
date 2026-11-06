//! Optical Kerning Pairs and Margin Protrusion (Hanging Punctuation) Tables.
//!
//! Models sub-em kerning adjustments in 1/1000 em units for diagonal/overhanging
//! Cyrillic and Latin letter pairs (`ТА`, `АВ`, `ГД`, `ГO`, `То`, `Тр`, `Уа`, `Yo`, `Wa`, `Av`)
//! and optical margin hang percentages (`microtype` protrusion) for punctuation.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KerningEvent {
    pub char_index: usize,
    pub left_char: char,
    pub right_char: char,
    /// Kerning adjustment in 1/1000 em units (negative = tighten, positive = open).
    pub adjustment_units: i16,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KerningSummary {
    pub total_pairs_inspected: usize,
    pub active_kern_pairs: usize,
    pub cumulative_tightening_units: i32,
    pub events: Vec<KerningEvent>,
}

/// Returns the optical kerning adjustment in 1/1000 em units for an adjacent `(left, right)`
/// glyph pair in a high-contrast Renaissance serif (such as Cormorant Garamond).
#[inline]
pub fn kern_pair_units(left: char, right: char) -> (i16, &'static str) {
    match (left, right) {
        // Cyrillic uppercase overhang + diagonal pairs
        ('Т', 'а') | ('Т', 'о') | ('Т', 'е') | ('Т', 'у') | ('Т', 'А') | ('Т', 'О') => {
            (-95, "Верхняя перекладина «Т» нависает над открытым овалом")
        }
        ('Г', 'а') | ('Г', 'о') | ('Г', 'е') | ('Г', 'у') | ('Г', 'д') | ('Г', 'А') | ('Г', 'О') => {
            (-110, "Глубокая оптическая пустота под консолью «Г»")
        }
        ('А', 'Т') | ('А', 'В') | ('А', 'Г') | ('А', 'У') | ('Д', 'А') | ('Л', 'Т') | ('Л', 'У') => {
            (-80, "Встречные диагонали образуют оптический провал")
        }
        ('Р', 'а') | ('Р', 'о') | ('Р', 'е') | ('Р', 'А') | ('У', 'а') | ('У', 'о') | ('У', 'А') => {
            (-75, "Нижнее полуполе «Р»/«У» требует компенсации")
        }
        ('В', 'а') | ('В', 'о') | ('К', 'о') | ('К', 'е') | ('К', 'а') | ('Х', 'о') | ('Х', 'е') => {
            (-45, "Согласование бокового апроша с круглым контуром")
        }
        ('г', 'о') | ('г', 'а') | ('г', 'е') | ('т', 'о') | ('т', 'а') | ('р', 'а') | ('р', 'о') => {
            (-35, "Строчное сближение открытого правого полуапроша")
        }

        // Latin classic kerning pairs
        ('T', 'o') | ('T', 'e') | ('T', 'a') | ('T', 'r') | ('T', 'u') | ('T', 'y') => {
            (-95, "T crossbar overhang compensation")
        }
        ('V', 'a') | ('V', 'o') | ('V', 'e') | ('W', 'a') | ('W', 'o') | ('Y', 'o') | ('Y', 'a') => {
            (-100, "Diagonal stem under-tuck")
        }
        ('A', 'V') | ('A', 'W') | ('A', 'T') | ('A', 'Y') | ('L', 'T') | ('L', 'V') | ('P', 'A') => {
            (-85, "Counter-diagonal capital pair")
        }

        // Guillemets and punctuation breathing room
        ('«', _) => (30, "Тонкая шпация после открывающей французской кавычки «ёлочки»"),
        (_, '»') => (30, "Тонкая шпация перед закрывающей кавычкой «ёлочкой»"),
        (_, '—') | ('—', _) => (40, "Воздух вокруг полиграфического длинного тире"),

        _ => (0, ""),
    }
}

/// Left margin optical protrusion in 1/1000 em units (how far a glyph at the start of a line
/// should hang into the left margin to preserve an optically straight vertical edge).
#[inline]
pub fn left_protrusion_units(ch: char) -> u16 {
    match ch {
        '«' | '“' | '"' | '\'' => 420,
        '(' | '[' => 300,
        'О' | 'С' | 'Э' | 'Ю' | 'O' | 'C' | 'G' | 'Q' => 65,
        'А' | 'Д' | 'Л' | 'A' | 'J' => 50,
        'Т' | 'Г' | 'У' | 'T' | 'V' | 'W' | 'Y' => 75,
        _ => 0,
    }
}

/// Right margin optical protrusion in 1/1000 em units (hanging hyphens, commas, periods,
/// and right guillemets into the right margin).
#[inline]
pub fn right_protrusion_units(ch: char) -> u16 {
    match ch {
        '.' | ',' => 550,
        '-' | '‐' => 500,
        ':' | ';' => 380,
        '»' | '”' | '"' | '\'' => 420,
        ')' | ']' | '!' | '?' => 240,
        '—' => 280,
        _ => 0,
    }
}

/// Scans a text and collects all active optical kerning pairs for typographic inspection.
pub fn analyze_text_kerning(text: &str) -> KerningSummary {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 {
        return KerningSummary {
            total_pairs_inspected: 0,
            active_kern_pairs: 0,
            cumulative_tightening_units: 0,
            events: Vec::new(),
        };
    }

    let mut events = Vec::new();
    let mut cumulative_tightening = 0i32;

    for i in 0..(chars.len() - 1) {
        let left = chars[i];
        let right = chars[i + 1];
        let (adj, reason) = kern_pair_units(left, right);
        if adj != 0 {
            cumulative_tightening += i32::from(adj);
            if events.len() < 32 {
                events.push(KerningEvent {
                    char_index: i,
                    left_char: left,
                    right_char: right,
                    adjustment_units: adj,
                    reason,
                });
            }
        }
    }

    KerningSummary {
        total_pairs_inspected: chars.len() - 1,
        active_kern_pairs: events.len(),
        cumulative_tightening_units: cumulative_tightening,
        events,
    }
}
