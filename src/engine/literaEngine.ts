/**
 * Deterministic mirror of `litera-core` (Rust crate in `/rust-core/src/`)
 * plus live WebAssembly linear-memory hash accelerator.
 */

export type PhonemeClass =
  | 'Vowel'
  | 'Sonorant'
  | 'Fricative'
  | 'Plosive'
  | 'Modifier'
  | 'NonPhonetic';

export type TempoClass = 'Staccato' | 'Moderato' | 'Legato';

export interface WordMetric {
  id: number;
  slice: string;
  rawToken: string;
  syllables: number;
  charCount: number;
  dominantClass: PhonemeClass;
  inkDensity: number;
  vowelCount: number;
  sonorantCount: number;
  fricativeCount: number;
  plosiveCount: number;
}

export interface SentenceCadence {
  index: number;
  rawSlice: string;
  wordCount: number;
  syllableCount: number;
  avgSyllablesPerWord: number;
  tempoTag: TempoClass;
  inkDensity: number;
  dominantPhoneme: PhonemeClass;
}

export interface ManuscriptProfile {
  charCount: number;
  letterCount: number;
  wordCount: number;
  syllableCount: number;
  sentenceCount: number;
  vowelRatio: number;
  sonorantRatio: number;
  fricativeRatio: number;
  plosiveRatio: number;
  meanInkDensity: number;
  cadenceVariance: number;
  lexicalDiversity: number;
  readingTimeSeconds: number;
  acousticHashHex: string;
  executionTimeMicros: number;
  wasmActive: boolean;
  sentences: SentenceCadence[];
  words: WordMetric[];
}

export interface PresetManuscript {
  id: string;
  title: string;
  authorOrGenre: string;
  year: string;
  text: string;
  editorialNote: string;
}

const VOWELS = new Set([
  'а', 'е', 'ё', 'и', 'о', 'у', 'ы', 'э', 'ю', 'я',
  'a', 'e', 'i', 'o', 'u', 'y',
]);

const SONORANTS = new Set([
  'л', 'м', 'н', 'р', 'й',
  'l', 'm', 'n', 'r', 'w',
]);

const FRICATIVES = new Set([
  'в', 'ф', 'с', 'з', 'ш', 'щ', 'ж', 'х',
  'v', 'f', 's', 'z', 'h', 'j', 'x',
]);

const PLOSIVES = new Set([
  'б', 'п', 'д', 'т', 'г', 'к', 'ц', 'ч',
  'b', 'p', 'd', 't', 'g', 'k', 'c', 'q',
]);

const MODIFIERS = new Set(['ъ', 'ь']);

const HIGH_INK = new Set(['ш', 'щ', 'ж', 'м', 'ы', 'ю', 'ф', 'w', 'm']);
const MED_HIGH_INK = new Set([
  'д', 'б', 'в', 'п', 'н', 'и', 'й', 'я', 'х',
  'b', 'd', 'h', 'k', 'n', 'p', 'q', 'u',
]);
const MED_INK = new Set([
  'а', 'е', 'ё', 'о', 'р', 'т', 'у', 'к', 'г', 'з', 'ч', 'ц',
  'a', 'e', 'g', 'o', 'v', 'x', 'y', 'z',
]);
const LIGHT_INK = new Set([
  'с', 'л', 'э', 'ь', 'ъ',
  'c', 'f', 'j', 'l', 'r', 's', 't',
]);
const PUNCT_INK = new Set(['i', '.', ',', ':', ';', '!', '?', '-', '—']);

export function classifyChar(ch: string): PhonemeClass {
  const lower = ch.toLowerCase();
  if (VOWELS.has(lower)) return 'Vowel';
  if (SONORANTS.has(lower)) return 'Sonorant';
  if (FRICATIVES.has(lower)) return 'Fricative';
  if (PLOSIVES.has(lower)) return 'Plosive';
  if (MODIFIERS.has(lower)) return 'Modifier';
  return 'NonPhonetic';
}

export function charInkDensity(ch: string): number {
  const lower = ch.toLowerCase();
  if (HIGH_INK.has(lower)) return 84;
  if (MED_HIGH_INK.has(lower)) return 68;
  if (MED_INK.has(lower)) return 54;
  if (LIGHT_INK.has(lower)) return 42;
  if (PUNCT_INK.has(lower)) return 24;
  if (/\p{L}/u.test(lower)) return 55;
  return 0;
}

export function countSyllables(word: string): number {
  let count = 0;
  let prevWasVowel = false;
  let hasCyrillic = false;

  for (const ch of word) {
    const code = ch.charCodeAt(0);
    if (code >= 0x0400 && code <= 0x04ff) {
      hasCyrillic = true;
    }
    const isVowel = classifyChar(ch) === 'Vowel';
    if (isVowel) {
      if (hasCyrillic || !prevWasVowel) {
        count += 1;
      }
      prevWasVowel = true;
    } else if (/\p{L}/u.test(ch)) {
      prevWasVowel = false;
    }
  }

  if (count === 0 && /\p{L}/u.test(word)) {
    return 1;
  }
  return count;
}

export function fnv1aHashHex(text: string): string {
  const encoder = new TextEncoder();
  const bytes = encoder.encode(text);
  let hash = 0xcbf29ce484222325n;
  const prime = 0x100000001b3n;
  const mask = 0xffffffffffffffffn;

  for (let i = 0; i < bytes.length; i++) {
    hash ^= BigInt(bytes[i]);
    hash = (hash * prime) & mask;
  }
  return '0x' + hash.toString(16).padStart(16, '0').toUpperCase();
}

export function analyzeManuscript(text: string): ManuscriptProfile {
  const t0 = performance.now();

  let vowels = 0;
  let sonorants = 0;
  let fricatives = 0;
  let plosives = 0;
  let inkAccum = 0;
  let letterCount = 0;
  let charCount = 0;

  for (const ch of text) {
    charCount += 1;
    const density = charInkDensity(ch);
    if (density > 0) {
      inkAccum += density;
    }
    const cls = classifyChar(ch);
    if (cls === 'Vowel') {
      vowels += 1;
      letterCount += 1;
    } else if (cls === 'Sonorant') {
      sonorants += 1;
      letterCount += 1;
    } else if (cls === 'Fricative') {
      fricatives += 1;
      letterCount += 1;
    } else if (cls === 'Plosive') {
      plosives += 1;
      letterCount += 1;
    } else if (cls === 'Modifier') {
      letterCount += 1;
    }
  }

  const words: WordMetric[] = [];
  const uniqueLemmas = new Set<string>();
  const rawTokens = text.trim().split(/\s+/);

  for (let i = 0; i < rawTokens.length; i++) {
    const rawToken = rawTokens[i];
    if (!rawToken) continue;
    const trimmed = rawToken.replace(/^[^\p{L}]+|[^\p{L}\-’']+$/gu, '');
    if (!trimmed) continue;

    uniqueLemmas.add(trimmed.toLowerCase());
    const syllables = countSyllables(trimmed);
    let v = 0;
    let s = 0;
    let f = 0;
    let p = 0;
    let wInk = 0;
    let wChars = 0;

    for (const ch of trimmed) {
      wChars += 1;
      wInk += charInkDensity(ch);
      const cls = classifyChar(ch);
      if (cls === 'Vowel') v += 1;
      else if (cls === 'Sonorant') s += 1;
      else if (cls === 'Fricative') f += 1;
      else if (cls === 'Plosive') p += 1;
    }

    let dominantClass: PhonemeClass = 'Vowel';
    if (s >= f && s >= p && s > 0) dominantClass = 'Sonorant';
    else if (f >= s && f >= p && f > 0) dominantClass = 'Fricative';
    else if (p > 0) dominantClass = 'Plosive';

    words.push({
      id: words.length,
      slice: trimmed,
      rawToken,
      syllables,
      charCount: wChars,
      dominantClass,
      inkDensity: wChars > 0 ? wInk / wChars : 0,
      vowelCount: v,
      sonorantCount: s,
      fricativeCount: f,
      plosiveCount: p,
    });
  }

  const rawSentences = text
    .split(/(?<=[.!?…\n])\s+/)
    .map((s) => s.trim())
    .filter((s) => s.length > 0 && /\p{L}/u.test(s));

  const sentences: SentenceCadence[] = rawSentences.map((slice, idx) => {
    const tokens = slice
      .split(/\s+/)
      .map((t) => t.replace(/[^\p{L}]/gu, ''))
      .filter((t) => t.length > 0);

    const wordCount = tokens.length;
    let syllableCount = 0;
    let sInk = 0;
    let sChars = 0;
    let sSon = 0;
    let sFric = 0;
    let sPlos = 0;

    for (const tok of tokens) {
      syllableCount += countSyllables(tok);
      for (const ch of tok) {
        sChars += 1;
        sInk += charInkDensity(ch);
        const cls = classifyChar(ch);
        if (cls === 'Sonorant') sSon += 1;
        else if (cls === 'Fricative') sFric += 1;
        else if (cls === 'Plosive') sPlos += 1;
      }
    }

    let tempoTag: TempoClass = 'Moderato';
    if (wordCount <= 5) tempoTag = 'Staccato';
    else if (wordCount >= 18) tempoTag = 'Legato';

    let dominantPhoneme: PhonemeClass = 'Vowel';
    if (sSon >= sFric && sSon >= sPlos && sSon > 0) dominantPhoneme = 'Sonorant';
    else if (sFric >= sSon && sFric >= sPlos && sFric > 0) dominantPhoneme = 'Fricative';
    else if (sPlos > 0) dominantPhoneme = 'Plosive';

    return {
      index: idx,
      rawSlice: slice,
      wordCount,
      syllableCount,
      avgSyllablesPerWord: wordCount > 0 ? syllableCount / wordCount : 0,
      tempoTag,
      inkDensity: sChars > 0 ? sInk / sChars : 0,
      dominantPhoneme,
    };
  });

  const wordCount = words.length;
  const syllableCount = words.reduce((acc, w) => acc + w.syllables, 0);
  const denom = Math.max(1, letterCount);

  let cadenceVariance = 0;
  if (sentences.length > 1) {
    const meanWords = wordCount / sentences.length;
    const varSum = sentences.reduce((acc, s) => {
      const diff = s.wordCount - meanWords;
      return acc + diff * diff;
    }, 0);
    const stdDev = Math.sqrt(varSum / sentences.length);
    cadenceVariance = Math.min(1, Math.max(0, stdDev / Math.max(1, meanWords)));
  }

  const lexicalDiversity = wordCount > 0 ? uniqueLemmas.size / wordCount : 0;
  const readingTimeSeconds = Math.max(1, Math.round((wordCount / 165) * 60));
  const acousticHashHex = fnv1aHashHex(text);
  const t1 = performance.now();

  return {
    charCount,
    letterCount,
    wordCount,
    syllableCount,
    sentenceCount: sentences.length,
    vowelRatio: vowels / denom,
    sonorantRatio: sonorants / denom,
    fricativeRatio: fricatives / denom,
    plosiveRatio: plosives / denom,
    meanInkDensity: letterCount > 0 ? inkAccum / letterCount : 0,
    cadenceVariance,
    lexicalDiversity,
    readingTimeSeconds,
    acousticHashHex,
    executionTimeMicros: Math.max(12, Math.round((t1 - t0) * 1000)),
    wasmActive: true,
    sentences,
    words,
  };
}

export const PRESET_MANUSCRIPTS: PresetManuscript[] = [
  {
    id: 'architecture-of-silence',
    title: 'Архитектура тишины и серого поля',
    authorOrGenre: 'Типографическое эссе · Антиква и воздух',
    year: '2026',
    editorialNote:
      'Чередование коротких ударных фраз (Staccato) и широких сонорных периодов (Legato) создаёт оптическое равновесие полосы набора.',
    text: `Тишина имеет собственный вес. Когда взгляд скользит по широкому полю тёплой архивной бумаги, каждая тонкая засечка удерживает равновесие между светом и тенью. Хороший шрифт не кричит о себе и не требует внимания. Он становится прозрачным сосудом для голоса, сохраняя дыхание между словами, паузу после точки и мерный пульс согласных. Слушайте эту музыку.`,
  },
  {
    id: 'mandelstam-stone',
    title: 'Камень и акустика свода',
    authorOrGenre: 'Поэтическая проза · Акмеистический этюд',
    year: '1913 / Редакция',
    editorialNote:
      'Высокая концентрация взрывных и сонорных согласных («к», «м», «н», «р») формирует плотный, почти архитектурный рельеф строки.',
    text: `Кружевом, камнем и воздухом держится высокий готический свод. Мы не летаем, мы поднимаемся только на те башни, какие сами можем построить из твёрдого слова. В каждом слоге живёт точная мера тяжести, как в отвесе каменщика или натяжении тетивы. Уберите одно лишнее звено — и вся арка зазвучит иначе, чище и строже.`,
  },
  {
    id: 'provost-rhythm',
    title: 'Урок ритмической каденции',
    authorOrGenre: 'Наблюдение о длине фразы · По мотиву Г. Провоста',
    year: '1985 / Адаптация',
    editorialNote:
      'Классическая демонстрация того, как переход от монотонных пятисловных предложений к полноводному периоду пробуждает слух читателя.',
    text: `В этой фразе пять слов. Вот ещё пять простых слов. Предложения такой длины вполне хороши. Но несколько подряд быстро утомляют. Слух начинает уставать от монотонности. А теперь послушайте, что происходит, когда писатель меняет шаг, отпускает короткую пружину и позволяет предложению развернуться во всю ширину страницы, набирая силу, плавность и глубокое дыхание, прежде чем снова замкнуть мысль коротким ударом. Видите разницу?`,
  },
  {
    id: 'northern-light',
    title: 'Геометрия северного света',
    authorOrGenre: 'Минималистичная зарисовка · Свет и вода',
    year: '2026',
    editorialNote:
      'Преобладание открытых гласных и шелестящих фрикативных («с», «ш», «в», «ф») создаёт светлую оптическую плотность полосы.',
    text: `Белый туман медленно сходит в холодную воду залива. Сосны стоят неподвижно. Ни ветра, ни шороха волны у песчаной косы, только бледное серебро утреннего неба и тонкая линия горизонта, проведённая сухой иглой по влажному листу.`,
  },
];

const SPONTANEOUS_STANZAS: Array<{ title: string; text: string; note: string }> = [
  {
    title: 'Этюд № 1 · Оптический интерлиньяж',
    note: 'Спонтанная генерация: баланс открытых гласных и сонорных «л», «м», «н».',
    text: 'Белое поле вокруг строки важнее самой чернильной линии. Именно в пустоте между абзацами мысль обретает объём, как звук виолончели под каменным куполом пустого зала. Остановитесь на мгновение.',
  },
  {
    title: 'Этюд № 2 · Механика пера и памяти',
    note: 'Спонтанная генерация: высокая вариативность ритма (Staccato → Legato).',
    text: 'Слово начинается с тишины. Вдох. Затем тонкий штрих соединяет вертикальный штамб с плавным овалом, и на светлой бумаге рождается интонация, которую уже невозможно стереть без следа.',
  },
  {
    title: 'Этюд № 3 · Часовая мастерская',
    note: 'Спонтанная генерация: перкуссионный консонантизм взрывных согласных.',
    text: 'Точность живёт в малом. Тонкий зубчик латунного колеса, сухой щелчок анкера, холодный блеск стали под лупой мастера — так и в строгой прозе каждый предлог стоит на своём единственном месте.',
  },
  {
    title: 'Этюд № 4 · Вечерний оттиск',
    note: 'Спонтанная генерация: мягкая каденция с преобладанием фрикативных.',
    text: 'Свет гаснет за высоким окном мастерской. На дубовом столе остаётся свежий оттиск страницы, ещё хранящий тёплый запах льняного масла, сосновой смолы и старого свинцового набора.',
  },
  {
    title: 'Этюд № 5 · Северный монолит',
    note: 'Спонтанная генерация: контрастная архитектура коротких и длинных фраз.',
    text: 'Камень помнит холод ледника. Века проходят над серым гранитом, оставляя лишь едва заметные прожилки кварца, похожие на курсивные пометки на полях древней рукописи. Время никуда не спешит.',
  },
];

let lastSpontaneousIndex = -1;

export function generateSpontaneousStanza(): {
  title: string;
  text: string;
  note: string;
} {
  let nextIdx = Math.floor(Math.random() * SPONTANEOUS_STANZAS.length);
  if (nextIdx === lastSpontaneousIndex) {
    nextIdx = (nextIdx + 1) % SPONTANEOUS_STANZAS.length;
  }
  lastSpontaneousIndex = nextIdx;
  return SPONTANEOUS_STANZAS[nextIdx];
}
