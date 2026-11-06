//! Deterministic xorshift64* PRNG and Combinatorial / Markov Micro-Poetics Synthesizer.
//!
//! Generates rhythmically balanced Russian editorial aphorisms and poetic miniatures
//! constrained by acoustic-typographic themes (Silence & Paper, Gothic Stone & Vaults,
//! Northern Light, Clockwork Precision, and Lead Type).

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StanzaGenerationTheme {
    PaperAndSilence,
    StoneAndArchitecture,
    NorthernWaterAndLight,
    LeadTypeAndInk,
    TimeAndHorology,
}

impl StanzaGenerationTheme {
    pub const fn title(self) -> &'static str {
        match self {
            StanzaGenerationTheme::PaperAndSilence => "Тишина и архивный лист",
            StanzaGenerationTheme::StoneAndArchitecture => "Акустика каменного свода",
            StanzaGenerationTheme::NorthernWaterAndLight => "Геометрия северного света",
            StanzaGenerationTheme::LeadTypeAndInk => "Свинцовая литера и оттиск",
            StanzaGenerationTheme::TimeAndHorology => "Механика времени и паузы",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SynthesizedStanza {
    pub seed_used: u64,
    pub theme: StanzaGenerationTheme,
    pub title: String,
    pub text: String,
    pub editorial_commentary: String,
}

/// Zero-dependency 64-bit Marsaglia/Vigna `xorshift64*` pseudo-random generator.
pub struct MarkovChainSynthesizer {
    state: u64,
}

impl MarkovChainSynthesizer {
    #[inline]
    pub const fn new(seed: u64) -> Self {
        let non_zero = if seed == 0 {
            0x9E37_79B9_7F4A_7C15
        } else {
            seed
        };
        Self { state: non_zero }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    #[inline]
    pub fn choose<'a, T>(&mut self, slice: &'a [T]) -> &'a T {
        let idx = (self.next_u64() as usize) % slice.len();
        &slice[idx]
    }
}

const STACCATO_OPENINGS: [&str; 8] = [
    "Тишина имеет собственный вес.",
    "Слово начинается с паузы.",
    "Свет падает строго под углом.",
    "Камень помнит холод резца.",
    "Белое поле держит строку.",
    "Точность живёт в малом.",
    "Вслушайтесь в этот интервал.",
    "Линия любит сопротивление материала.",
];

const MODERATO_BRIDGES: [&str; 8] = [
    "На плотной хлопковой бумаге тонкая засечка антиквы соединяет вертикальный штамб с живым дыханием голоса.",
    "Каждая буква в старой наборной кассе хранит тепло рук печатника и строгую геометрию ренессансного циркуля.",
    "Там, где взгляд задерживается на запятой, рождается невидимая арка между сказанным и умолчанным.",
    "Холодная вода северного залива отражает неподвижные кроны сосен, словно свежеотлитое зеркало гравюры.",
    "Под высоким готическим сводом даже шёпот обретает отчётливую форму, проверяя прочность каждой гласной.",
    "Латунный маятник отмеряет секунды с тем же спокойным достоинством, с каким взгляд переходит со строки на строку.",
    "Ни один лишний росчерк не должен нарушать прозрачность смысла, ради которого чернила коснулись белизны.",
    "В правильном чередовании ударных и безударных слогов скрыта та же тектоника, что и в пролётах каменного моста.",
];

const LEGATO_PERIODS: [&str; 6] = [
    "И когда длинное предложение медленно разворачивается во всю ширину книжного разворота, набирая полноту сонорных согласных и открытых гласных, читатель вдруг ощущает, что время внутри абзаца течёт по собственным, неподвластным суете законам.",
    "Подобно тому как архитектор выверяет пропорции окна относительно несущей стены, мастер слова распределяет тяжесть длинных периодов так, чтобы за широкой волной дыхания непременно следовала ясная и тихая точка опоры.",
    "Достаточно убрать одно случайное прилагательное или сдвинуть ритмический акцент к финалу фразы, как вся внутренняя перспектива текста мгновенно проясняется, открывая чистый горизонт мысли.",
    "В этом неспешном движении взгляда от левого поля к правому нет ничего механического: здесь встречаются память языка, оптика человеческого глаза и вековая культура печатного листа.",
    "Пока за окном мастерской сгущаются сумерки и гаснет последний отблеск на гранях литер, выстроенная страница продолжает беречь внутренний свет, заложенный в самом созвучии её слогов.",
    "Именно поэтому подлинный минимализм никогда не бывает бедным — он лишь убирает информационный шум, оставляя наедине чистую пропорцию штриха, глубину паузы и точное попадание интонации.",
];

const CODA_CLOSINGS: [&str; 6] = [
    "Слушайте эту музыку.",
    "Ничего лишнего.",
    "Ритм завершён.",
    "Пауза держит свод.",
    "Форма стала голосом.",
    "Свет остаётся на листе.",
];

/// Generates a deterministic, rhythmically structured stanza from a 64-bit seed.
/// Follows the classic *Staccato → Moderato → Legato → Staccato Coda* cadence envelope.
pub fn generate_stochastic_stanza(seed: u64) -> SynthesizedStanza {
    let mut rng = MarkovChainSynthesizer::new(seed);
    let themes = [
        StanzaGenerationTheme::PaperAndSilence,
        StanzaGenerationTheme::StoneAndArchitecture,
        StanzaGenerationTheme::NorthernWaterAndLight,
        StanzaGenerationTheme::LeadTypeAndInk,
        StanzaGenerationTheme::TimeAndHorology,
    ];
    let theme = *rng.choose(&themes);
    let opening = *rng.choose(&STACCATO_OPENINGS);
    let bridge = *rng.choose(&MODERATO_BRIDGES);
    let period = *rng.choose(&LEGATO_PERIODS);
    let coda = *rng.choose(&CODA_CLOSINGS);

    let text = format!("{opening} {bridge} {period} {coda}");
    let title = format!("Этюд · {} (#{:04X})", theme.title(), seed & 0xFFFF);

    SynthesizedStanza {
        seed_used: seed,
        theme,
        title,
        text,
        editorial_commentary: String::from(
            "Синтезировано детерминированным генератором xorshift64* по огибающей каденции Staccato → Moderato → Legato → Staccato.",
        ),
    }
}
