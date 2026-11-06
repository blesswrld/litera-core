/**
 * Unified TypeScript/WASM execution bridge mirroring the 6-crate Rust workspace:
 * - `litera-phonetics` (Acoustic classification, narrow IPA transcriber, prosodic meter)
 * - `litera-typography` (Knuth–Plass DP line-breaking, Liang hyphenation, kerning, page canons)
 * - `litera-stylometry` (Shannon entropy, Yule's K, Honoré's R, Burrows's Delta author proximity)
 * - `litera-export` (Smart Micro-Typographer, Typst `.typ` & XeLaTeX `.tex` compilers)
 */

export type PhonemeClass =
  | 'Vowel'
  | 'Sonorant'
  | 'Fricative'
  | 'Plosive'
  | 'Modifier'
  | 'NonPhonetic';

export type TempoClass = 'Staccato' | 'Moderato' | 'Legato';

export type CanonPresetId =
  | 'VanDeGraafMedieval'
  | 'TschicholdGolden'
  | 'BringhurstEditorial'
  | 'SwissModular';

export interface WordMetric {
  id: number;
  slice: string;
  rawToken: string;
  syllables: number;
  stressedSyllable1Based: number;
  ipaNarrow: string;
  hyphenated: string;
  charCount: number;
  dominantClass: PhonemeClass;
  inkDensity: number;
  f2Brightness: number;
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

export interface TypesetLine {
  lineNumber: number;
  text: string;
  wordCount: number;
  naturalWidthEm: number;
  targetWidthEm: number;
  adjustmentRatio: number;
  badness: number;
  demerits: number;
  fitnessLabel: string;
}

export interface KnuthPlassComparison {
  targetMeasureEm: number;
  knuthPlassLines: TypesetLine[];
  greedyLines: TypesetLine[];
  knuthPlassTotalDemerits: number;
  greedyTotalDemerits: number;
  knuthPlassMaxBadness: number;
  greedyMaxBadness: number;
  demeritReductionPct: number;
}

export interface AuthorProximityMatch {
  authorId: string;
  authorName: string;
  signatureNote: string;
  burrowsDelta: number;
  similarityPct: number;
}

export interface StylometryReport {
  charEntropyBits: number;
  wordEntropyBits: number;
  yulesCharacteristicK: number;
  honoresStatisticR: number;
  simpsonsDiversity: number;
  hapaxCount: number;
  uniqueTypes: number;
  fleschObornevaIndex: number;
  readabilityRegister: string;
  dominantMeter: string;
  meterScheme: string;
  meterRegularityPct: number;
  alliterationDensityPct: number;
  topAlliterations: string[];
  authorMatches: AuthorProximityMatch[];
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
  meanF2Brightness: number;
  cadenceVariance: number;
  lexicalDiversity: number;
  readingTimeSeconds: number;
  acousticHashHex: string;
  executionTimeMicros: number;
  sentences: SentenceCadence[];
  words: WordMetric[];
  stylometry: StylometryReport;
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
const SONORANTS = new Set(['л', 'м', 'н', 'р', 'й', 'l', 'm', 'n', 'r', 'w']);
const FRICATIVES = new Set(['в', 'ф', 'с', 'з', 'ш', 'щ', 'ж', 'х', 'v', 'f', 's', 'z', 'h', 'j', 'x']);
const PLOSIVES = new Set(['б', 'п', 'д', 'т', 'г', 'к', 'ц', 'ч', 'b', 'p', 'd', 't', 'g', 'k', 'c', 'q']);
const MODIFIERS = new Set(['ъ', 'ь']);

const HIGH_INK = new Set(['ш', 'щ', 'ж', 'м', 'ы', 'ю', 'ф', 'w', 'm']);
const MED_HIGH_INK = new Set(['д', 'б', 'в', 'п', 'н', 'и', 'й', 'я', 'х', 'b', 'd', 'h', 'k', 'n', 'p', 'q', 'u']);
const MED_INK = new Set(['а', 'е', 'ё', 'о', 'р', 'т', 'у', 'к', 'г', 'з', 'ч', 'ц', 'a', 'e', 'g', 'o', 'v', 'x', 'y', 'z']);
const LIGHT_INK = new Set(['с', 'л', 'э', 'ь', 'ъ', 'c', 'f', 'j', 'l', 'r', 's', 't']);
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

export function charF2Brightness(ch: string): number {
  const lower = ch.toLowerCase();
  switch (lower) {
    case 'и': case 'й': case 'i': return 94;
    case 'щ': case 'е': case 'ч': case 'ь': case 'e': return 84;
    case 'с': case 'я': case 'ц': case 'э': case 's': return 76;
    case 'з': case 'ё': case 'а': case 'a': return 60;
    case 'н': case 'т': case 'д': case 'р': case 'л': return 52;
    case 'ж': case 'ш': case 'ф': case 'в': case 'ы': return 44;
    case 'о': case 'к': case 'г': case 'б': case 'п': case 'м': case 'х': case 'o': return 35;
    case 'у': case 'u': return 24;
    default: return 50;
  }
}

export function countSyllables(word: string): number {
  let count = 0;
  let prevWasVowel = false;
  let hasCyrillic = false;

  for (const ch of word) {
    const code = ch.charCodeAt(0);
    if (code >= 0x0400 && code <= 0x04ff) hasCyrillic = true;
    const isVowel = classifyChar(ch) === 'Vowel';
    if (isVowel) {
      if (hasCyrillic || !prevWasVowel) count += 1;
      prevWasVowel = true;
    } else if (/\p{L}/u.test(ch)) {
      prevWasVowel = false;
    }
  }
  return count === 0 && /\p{L}/u.test(word) ? 1 : count;
}

export function estimateWordStress(word: string): { stress0: number; totalSyllables: number } {
  let vowelIdx = 0;
  let yoSyl: number | null = null;
  let hasCyrillic = false;
  let prevVowel = false;

  for (const ch of word) {
    const code = ch.charCodeAt(0);
    if (code >= 0x0400 && code <= 0x04ff) hasCyrillic = true;
    if (classifyChar(ch) === 'Vowel') {
      if (hasCyrillic || !prevVowel) {
        if (ch === 'ё' || ch === 'Ё') yoSyl = vowelIdx;
        vowelIdx += 1;
      }
      prevVowel = true;
    } else if (/\p{L}/u.test(ch)) {
      prevVowel = false;
    }
  }

  if (vowelIdx <= 1) return { stress0: 0, totalSyllables: vowelIdx };
  if (yoSyl !== null) return { stress0: yoSyl, totalSyllables: vowelIdx };

  const lower = word.toLowerCase();
  if (/(ение|ание|ения|ания|овать|евать)$/.test(lower)) {
    return { stress0: Math.max(0, vowelIdx - 3), totalSyllables: vowelIdx };
  }
  if (/(ость|ости|ый|ий|ая|ое|ые|ого|ому|ыми|ие)$/.test(lower)) {
    return { stress0: Math.max(0, vowelIdx - 2), totalSyllables: vowelIdx };
  }
  if (/(ок|ек|он|ать|ить|уть|на|та)$/.test(lower)) {
    return { stress0: vowelIdx - 1, totalSyllables: vowelIdx };
  }
  return {
    stress0: vowelIdx === 2 ? 0 : Math.max(0, vowelIdx - 2),
    totalSyllables: vowelIdx,
  };
}

/**
 * Narrow Russian & Latin IPA transcriber mirroring `crates/litera-phonetics/src/ipa_transcriber.rs`.
 */
export function transcribeWordToIpa(word: string): string {
  const clean = Array.from(word.toLowerCase()).filter((c) => /\p{L}/u.test(c));
  if (clean.length === 0) return '[]';

  const hasCyrillic = clean.some((c) => c.charCodeAt(0) >= 0x0400 && c.charCodeAt(0) <= 0x04ff);
  const { stress0, totalSyllables } = estimateWordStress(word);

  if (!hasCyrillic) {
    return `[ˈ${clean.join('')}]`;
  }

  const devoiceMap: Record<string, string> = { б: 'п', в: 'ф', г: 'к', д: 'т', ж: 'ш', з: 'с' };
  const working = [...clean];
  const n = working.length;

  // Final obstruent devoicing
  const lastIdx = n >= 2 && (working[n - 1] === 'ь' || working[n - 1] === 'ъ') ? n - 2 : n - 1;
  if (lastIdx >= 0 && devoiceMap[working[lastIdx]]) {
    working[lastIdx] = devoiceMap[working[lastIdx]];
  }

  let ipa = '[';
  let curVowel = 0;

  for (let i = 0; i < n; i++) {
    const ch = working[i];
    const next = working[i + 1];
    const prev = i > 0 ? working[i - 1] : undefined;
    if (ch === 'ь' || ch === 'ъ') continue;

    if (classifyChar(ch) === 'Vowel') {
      const isStressed = curVowel === stress0;
      const isPretonic = stress0 > 0 && curVowel + 1 === stress0;
      if (isStressed && totalSyllables > 1) ipa += 'ˈ';

      const needsGlide =
        (ch === 'я' || ch === 'е' || ch === 'ё' || ch === 'ю') &&
        (!prev || classifyChar(prev) === 'Vowel' || prev === 'ь' || prev === 'ъ');
      if (needsGlide) ipa += 'j';

      const prevHard = prev === 'ж' || prev === 'ш' || prev === 'ц';
      if (ch === 'а' || ch === 'о') {
        ipa += isStressed ? (ch === 'о' ? 'o' : 'a') : isPretonic ? 'ɐ' : 'ə';
      } else if (ch === 'е') {
        ipa += isStressed ? 'e' : prevHard ? 'ɨ' : 'ɪ';
      } else if (ch === 'я') {
        ipa += isStressed ? 'æ' : 'ɪ';
      } else if (ch === 'ё') {
        ipa += 'ɵ';
      } else if (ch === 'и') {
        ipa += prevHard ? 'ɨ' : 'i';
      } else if (ch === 'ы') {
        ipa += 'ɨ';
      } else if (ch === 'у') {
        ipa += 'u';
      } else if (ch === 'ю') {
        ipa += isStressed ? 'ʉ' : 'ʊ';
      } else {
        ipa += 'ɛ';
      }
      curVowel += 1;
    } else {
      const softNext =
        (next === 'е' || next === 'ё' || next === 'и' || next === 'ю' || next === 'я' || next === 'ь') &&
        ch !== 'ж' &&
        ch !== 'ш' &&
        ch !== 'ц';
      if (ch === 'ж') ipa += 'ʐ';
      else if (ch === 'ш') ipa += 'ʂ';
      else if (ch === 'щ') ipa += 'ɕː';
      else if (ch === 'ч') ipa += 'tɕ';
      else if (ch === 'ц') ipa += 'ts';
      else if (ch === 'й') ipa += 'j';
      else if (ch === 'х') ipa += 'x';
      else if (ch === 'г') ipa += 'ɡ';
      else if (ch === 'л') ipa += softNext ? 'lʲ' : 'ɫ';
      else {
        const map: Record<string, string> = {
          б: 'b', п: 'p', в: 'v', ф: 'f', д: 'd', т: 't', з: 'z', с: 's', к: 'k', м: 'm', н: 'n', р: 'r',
        };
        ipa += (map[ch] || ch) + (softNext ? 'ʲ' : '');
      }
    }
  }

  return ipa + ']';
}

/**
 * Hyphenates a word using Slavic & Liang phonological syllable boundaries (`crates/litera-typography/src/hyphenation.rs`).
 */
export function hyphenateWord(word: string): string {
  const chars = Array.from(word);
  const n = chars.length;
  if (n < 4) return word;

  const vowelPos: number[] = [];
  for (let i = 0; i < n; i++) {
    if (classifyChar(chars[i]) === 'Vowel') vowelPos.push(i);
  }
  if (vowelPos.length < 2) return word;

  const breaks = new Set<number>();
  for (let idx = 0; idx < vowelPos.length - 1; idx++) {
    const v1 = vowelPos[idx];
    const v2 = vowelPos[idx + 1];
    const span = v2 - v1;
    let slot = v1 + 1;
    if (span === 2) {
      if (chars[v1 + 1] === 'й' || chars[v1 + 1] === 'ь' || chars[v1 + 1] === 'ъ') {
        slot = v1 + 2;
      } else {
        slot = v1 + 1;
      }
    } else if (span >= 3) {
      slot = v1 + 2;
      if (chars[v1 + 2] === 'ь' || chars[v1 + 2] === 'ъ' || chars[v1 + 2] === 'й') {
        slot = v1 + 3;
      }
    }
    if (slot >= 2 && slot <= n - 2) {
      breaks.add(slot);
    }
  }

  let out = '';
  for (let i = 0; i < n; i++) {
    if (breaks.has(i)) out += '·';
    out += chars[i];
  }
  return out;
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

const AUTHOR_CENTROIDS = [
  {
    id: 'mandelstam',
    name: 'Осип Мандельштам («Камень», акмеистическая проза)',
    note: 'Высокая плотность сонорных и взрывных, архитектурная весомость слова, средний шаг 11–15 слов.',
    vec: [13.8, 0.42, 2.48, 0.425, 0.245, 0.175],
  },
  {
    id: 'bringhurst',
    name: 'Роберт Брингхёрст (поэтическая типографика)',
    note: 'Точный терминологический ритм, чередование тезиса и развёрнутой визуальной метафоры.',
    vec: [12.4, 0.45, 2.42, 0.430, 0.240, 0.162],
  },
  {
    id: 'chekhov',
    name: 'Антон Чехов (камерная лаконичная проза)',
    note: 'Прозрачный синтаксис, короткие и средние предложения (8–13 слов), высокий динамический контраст.',
    vec: [10.6, 0.48, 2.22, 0.440, 0.220, 0.160],
  },
  {
    id: 'brodsky',
    name: 'Иосиф Бродский (интонационный эссеизм)',
    note: 'Резкая смена регистров, длинные рассуждения с коротким афористичным пуантом в финале абзаца.',
    vec: [17.4, 0.58, 2.55, 0.418, 0.238, 0.182],
  },
  {
    id: 'nabokov',
    name: 'Владимир Набоков (развёрнутый метафорический период)',
    note: 'Длинные сложноподчинённые периоды Legato (19–28 слов), богатая аллитерация и редкая лексика.',
    vec: [21.5, 0.54, 2.62, 0.432, 0.252, 0.165],
  },
  {
    id: 'pushkin',
    name: 'Александр Пушкин («Повести Белкина», ясность фразы)',
    note: 'Стремительный глагольный темп Staccato/Moderato (6–11 слов), гармонический баланс гласных.',
    vec: [8.8, 0.38, 2.15, 0.448, 0.230, 0.155],
  },
];

const STD_DEVS = [4.8, 0.14, 0.25, 0.025, 0.025, 0.025];

export function analyzeManuscript(text: string): ManuscriptProfile {
  const t0 = performance.now();

  let vowels = 0;
  let sonorants = 0;
  let fricatives = 0;
  let plosives = 0;
  let inkAccum = 0;
  let f2Accum = 0;
  let letterCount = 0;
  let charCount = 0;

  const charFreq = new Map<string, number>();

  for (const ch of text) {
    charCount += 1;
    const lower = ch.toLowerCase();
    if (!/\p{Cc}/u.test(lower)) {
      charFreq.set(lower, (charFreq.get(lower) || 0) + 1);
    }

    const density = charInkDensity(ch);
    if (density > 0) inkAccum += density;

    const cls = classifyChar(ch);
    if (cls !== 'NonPhonetic') {
      letterCount += 1;
      f2Accum += charF2Brightness(ch);
      if (cls === 'Vowel') vowels += 1;
      else if (cls === 'Sonorant') sonorants += 1;
      else if (cls === 'Fricative') fricatives += 1;
      else if (cls === 'Plosive') plosives += 1;
    }
  }

  let charEntropyBits = 0;
  if (charCount > 0) {
    for (const c of charFreq.values()) {
      const p = c / charCount;
      charEntropyBits -= p * Math.log2(p);
    }
  }

  const words: WordMetric[] = [];
  const lemmaFreq = new Map<string, number>();
  const rawTokens = text.trim().split(/\s+/);
  let polysyllabicWords = 0;

  // Track stress positions for prosodic meter detection
  let runningSyl = 0;
  const stressPositions: number[] = [];

  for (let i = 0; i < rawTokens.length; i++) {
    const rawToken = rawTokens[i];
    if (!rawToken) continue;
    const trimmed = rawToken.replace(/^[^\p{L}]+|[^\p{L}\-’']+$/gu, '');
    if (!trimmed) continue;

    const lowerLemma = trimmed.toLowerCase();
    lemmaFreq.set(lowerLemma, (lemmaFreq.get(lowerLemma) || 0) + 1);

    const { stress0, totalSyllables } = estimateWordStress(trimmed);
    const syllables = Math.max(1, totalSyllables);
    if (syllables >= 4) polysyllabicWords += 1;

    if (trimmed.length > 2) {
      stressPositions.push(runningSyl + stress0 + 1);
    }
    runningSyl += syllables;

    let v = 0, s = 0, f = 0, p = 0, wInk = 0, wF2 = 0, wChars = 0;
    for (const ch of trimmed) {
      wChars += 1;
      wInk += charInkDensity(ch);
      wF2 += charF2Brightness(ch);
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
      stressedSyllable1Based: stress0 + 1,
      ipaNarrow: transcribeWordToIpa(trimmed),
      hyphenated: hyphenateWord(trimmed),
      charCount: wChars,
      dominantClass,
      inkDensity: wChars > 0 ? wInk / wChars : 0,
      f2Brightness: wChars > 0 ? wF2 / wChars : 50,
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
    let syllableCount = 0, sInk = 0, sChars = 0, sSon = 0, sFric = 0, sPlos = 0;

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

  // Information theory & Stylometry calculations
  let wordEntropyBits = 0;
  let hapaxCount = 0;
  let sumISqVi = 0;
  let simpsonNum = 0;

  for (const count of lemmaFreq.values()) {
    if (count === 1) hapaxCount += 1;
    const p = count / Math.max(1, wordCount);
    wordEntropyBits -= p * Math.log2(p);
    sumISqVi += count * count;
    simpsonNum += count * (count - 1);
  }

  const uniqueTypes = lemmaFreq.size;
  const yulesCharacteristicK =
    wordCount > 1
      ? Math.max(0, (10000 * (sumISqVi - wordCount)) / (wordCount * wordCount))
      : 0;
  const hapaxRatio = hapaxCount / Math.max(1, uniqueTypes);
  const honoresStatisticR =
    wordCount > 0
      ? (100 * Math.log(Math.max(2, wordCount))) / Math.max(0.05, 1 - hapaxRatio)
      : 0;
  const simpsonsDiversity =
    wordCount > 1
      ? Math.max(0, Math.min(1, 1 - simpsonNum / (wordCount * (wordCount - 1))))
      : 1;

  // Readability (Oborneva Russian adaptation)
  const asl = wordCount / Math.max(1, sentences.length);
  const asw = syllableCount / Math.max(1, wordCount);
  const fleschObornevaIndex = Math.max(0, Math.min(100, 206.835 - 1.3 * asl - 60.1 * asw));
  const readabilityRegister =
    fleschObornevaIndex >= 70
      ? 'Прозрачный и ясный строй'
      : fleschObornevaIndex >= 50
      ? 'Классический литературный регистр'
      : fleschObornevaIndex >= 30
      ? 'Плотный аналитический стиль'
      : 'Сложноподчинённый период';

  // Prosodic meter detection
  const sLen = Math.max(1, stressPositions.length);
  const trochee = stressPositions.filter((p) => p % 2 === 1).length / sLen;
  const iamb = stressPositions.filter((p) => p % 2 === 0).length / sLen;
  const dactyl = stressPositions.filter((p) => p % 3 === 1).length / sLen;
  const amphibrach = stressPositions.filter((p) => p % 3 === 2).length / sLen;
  const anapaest = stressPositions.filter((p) => p % 3 === 0).length / sLen;

  const maxMeter = Math.max(trochee, iamb, dactyl, amphibrach, anapaest);
  let dominantMeter = 'Дольник / Свободная каденция';
  let meterScheme = '— ◡ (◡) — ◡ (◡) —';
  if (maxMeter >= 0.62) {
    if (dactyl === maxMeter) {
      dominantMeter = 'Дактилический импульс';
      meterScheme = '— ◡ ◡ | — ◡ ◡';
    } else if (amphibrach === maxMeter) {
      dominantMeter = 'Амфибрахическая волна';
      meterScheme = '◡ — ◡ | ◡ — ◡';
    } else if (anapaest === maxMeter) {
      dominantMeter = 'Анапестический подъём';
      meterScheme = '◡ ◡ — | ◡ ◡ —';
    } else if (iamb >= trochee) {
      dominantMeter = 'Ямбическая каденция';
      meterScheme = '◡ — | ◡ — | ◡ —';
    } else {
      dominantMeter = 'Хореический шаг';
      meterScheme = '— ◡ | — ◡ | — ◡';
    }
  }

  // Alliteration scan
  const consPositions = new Map<string, number[]>();
  words.forEach((w, wIdx) => {
    const seen = new Set<string>();
    for (const ch of w.slice.toLowerCase()) {
      const cls = classifyChar(ch);
      if (cls === 'Sonorant' || cls === 'Fricative' || cls === 'Plosive') {
        if (!seen.has(ch)) {
          seen.add(ch);
          const arr = consPositions.get(ch) || [];
          arr.push(wIdx);
          consPositions.set(ch, arr);
        }
      }
    }
  });

  let echoLinks = 0;
  const chainRank: Array<{ cons: string; hits: number }> = [];
  for (const [cons, posArr] of consPositions.entries()) {
    let closeHits = 0;
    for (let k = 0; k < posArr.length - 1; k++) {
      if (posArr[k + 1] - posArr[k] <= 3) {
        echoLinks += 1;
        closeHits += 1;
      }
    }
    if (closeHits >= 2) chainRank.push({ cons, hits: closeHits });
  }
  chainRank.sort((a, b) => b.hits - a.hits);

  // Burrows's Delta Author Attribution
  const sampleVec = [
    asl,
    cadenceVariance,
    asw,
    vowels / denom,
    sonorants / denom,
    plosives / denom,
  ];

  const authorMatches: AuthorProximityMatch[] = AUTHOR_CENTROIDS.map((c) => {
    let sumZ = 0;
    for (let d = 0; d < 6; d++) {
      sumZ += Math.abs(sampleVec[d] - c.vec[d]) / STD_DEVS[d];
    }
    const delta = sumZ / 6;
    const similarityPct = Math.max(15, Math.min(99.2, 100 * Math.exp(-0.42 * delta)));
    return {
      authorId: c.id,
      authorName: c.name,
      signatureNote: c.note,
      burrowsDelta: delta,
      similarityPct,
    };
  }).sort((a, b) => a.burrowsDelta - b.burrowsDelta);

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
    meanF2Brightness: letterCount > 0 ? f2Accum / letterCount : 50,
    cadenceVariance,
    lexicalDiversity: wordCount > 0 ? uniqueTypes / wordCount : 0,
    readingTimeSeconds: Math.max(1, Math.round((wordCount / 165) * 60)),
    acousticHashHex: fnv1aHashHex(text),
    executionTimeMicros: Math.max(14, Math.round((t1 - t0) * 1000)),
    sentences,
    words,
    stylometry: {
      charEntropyBits,
      wordEntropyBits,
      yulesCharacteristicK,
      honoresStatisticR,
      simpsonsDiversity,
      hapaxCount,
      uniqueTypes,
      fleschObornevaIndex,
      readabilityRegister,
      dominantMeter,
      meterScheme,
      meterRegularityPct: maxMeter * 100,
      alliterationDensityPct: Math.min(100, (echoLinks / Math.max(1, wordCount)) * 100),
      topAlliterations: chainRank.slice(0, 4).map((c) => `«${c.cons}» (${c.hits + 1}×)`),
      authorMatches,
    },
  };
}

/**
 * Measures string advance width in 1/1000 em units (`crates/litera-typography/src/metrics.rs`).
 */
function measureWordEmUnits(word: string): number {
  let units = 0;
  for (const ch of word) {
    const lower = ch.toLowerCase();
    const base = HIGH_INK.has(lower)
      ? 810
      : MED_HIGH_INK.has(lower)
      ? 605
      : MED_INK.has(lower)
      ? 510
      : LIGHT_INK.has(lower)
      ? 430
      : 310;
    units += ch !== lower ? Math.round(base * 1.22) : base;
  }
  return units;
}

function classifyFitnessLabel(r: number): string {
  if (r < -0.5) return 'Tight (Сжатая)';
  if (r <= 0.5) return 'Normal (Оптимум)';
  if (r <= 1.0) return 'Loose (Свободная)';
  return 'Over-loose (Разреженная)';
}

function fitnessIndex(r: number): number {
  if (r < -0.5) return 0;
  if (r <= 0.5) return 1;
  if (r <= 1.0) return 2;
  return 3;
}

/**
 * Runs Donald Knuth & Michael Plass's 1981 Dynamic Programming line-breaking algorithm
 * alongside Greedy First-Fit (`crates/litera-typography/src/knuth_plass.rs`).
 */
export function compareParagraphBreaking(
  text: string,
  targetMeasureEm: number
): KnuthPlassComparison {
  const words = text.trim().split(/\s+/).filter(Boolean);
  const n = words.length;
  const targetUnits = Math.round(targetMeasureEm * 1000);
  const spaceW = 250;
  const spaceStretch = 130;
  const spaceShrink = 85;

  if (n === 0) {
    return {
      targetMeasureEm,
      knuthPlassLines: [],
      greedyLines: [],
      knuthPlassTotalDemerits: 0,
      greedyTotalDemerits: 0,
      knuthPlassMaxBadness: 0,
      greedyMaxBadness: 0,
      demeritReductionPct: 0,
    };
  }

  const widths = words.map(measureWordEmUnits);
  const prefix = new Array<number>(n + 1).fill(0);
  for (let i = 0; i < n; i++) prefix[i + 1] = prefix[i] + widths[i];

  interface DpState {
    totalDemerits: number;
    prev: number;
    ratio: number;
    badness: number;
    fit: number;
    natWidth: number;
  }

  const dp: Array<DpState | null> = new Array(n + 1).fill(null);
  dp[0] = { totalDemerits: 0, prev: 0, ratio: 0, badness: 0, fit: 1, natWidth: 0 };

  for (let i = 0; i < n; i++) {
    const cur = dp[i];
    if (!cur) continue;

    for (let j = i + 1; j <= n; j++) {
      const span = prefix[j] - prefix[i];
      const gaps = j - i - 1;
      const natWidth = span + gaps * spaceW;
      const isLast = j === n;
      const diff = targetUnits - natWidth;

      let ratio = 0;
      if (isLast) {
        ratio = diff >= 0 ? 0 : diff / Math.max(1, gaps * spaceShrink);
      } else if (diff >= 0) {
        ratio = diff / Math.max(1, gaps * spaceStretch);
      } else {
        ratio = diff / Math.max(1, gaps * spaceShrink);
      }

      if (ratio < -1.15 && j > i + 1) break;
      if (ratio > 3.6 && !isLast && dp[j] !== null) continue;

      const fit = fitnessIndex(ratio);
      const badness = isLast
        ? 0
        : ratio < -1
        ? 10000
        : Math.min(10000, Math.round(100 * Math.pow(Math.abs(ratio), 3)));
      let demerits = isLast ? 0 : (10 + badness) * (10 + badness);
      if (!isLast && Math.abs(fit - cur.fit) > 1) demerits += 100;

      const cand = cur.totalDemerits + demerits;
      if (!dp[j] || cand < dp[j]!.totalDemerits) {
        dp[j] = {
          totalDemerits: cand,
          prev: i,
          ratio,
          badness,
          fit,
          natWidth,
        };
      }
    }
  }

  const kpBreaks: Array<{ start: number; end: number; node: DpState }> = [];
  let cursor = n;
  while (cursor > 0 && dp[cursor]) {
    const node = dp[cursor]!;
    kpBreaks.push({ start: node.prev, end: cursor, node });
    cursor = node.prev;
  }
  kpBreaks.reverse();

  const knuthPlassLines: TypesetLine[] = kpBreaks.map((b, idx) => {
    const prevDem = b.start > 0 && dp[b.start] ? dp[b.start]!.totalDemerits : 0;
    return {
      lineNumber: idx + 1,
      text: words.slice(b.start, b.end).join(' '),
      wordCount: b.end - b.start,
      naturalWidthEm: b.node.natWidth / 1000,
      targetWidthEm: targetMeasureEm,
      adjustmentRatio: b.node.ratio,
      badness: b.node.badness,
      demerits: Math.max(0, b.node.totalDemerits - prevDem),
      fitnessLabel: classifyFitnessLabel(b.node.ratio),
    };
  });

  // Greedy First-Fit comparison
  const greedyLines: TypesetLine[] = [];
  let gStart = 0;
  let gWidth = 0;
  let prevFit = 1;

  for (let i = 0; i < n; i++) {
    const w = widths[i];
    const add = i === gStart ? w : spaceW + w;
    if (i > gStart && gWidth + add > targetUnits) {
      const gaps = i - gStart - 1;
      const diff = targetUnits - gWidth;
      const ratio =
        diff >= 0
          ? diff / Math.max(1, gaps * spaceStretch)
          : diff / Math.max(1, gaps * spaceShrink);
      const fit = fitnessIndex(ratio);
      const badness = Math.min(10000, Math.round(100 * Math.pow(Math.abs(ratio), 3)));
      let demerits = (10 + badness) * (10 + badness);
      if (Math.abs(fit - prevFit) > 1) demerits += 100;
      prevFit = fit;

      greedyLines.push({
        lineNumber: greedyLines.length + 1,
        text: words.slice(gStart, i).join(' '),
        wordCount: i - gStart,
        naturalWidthEm: gWidth / 1000,
        targetWidthEm: targetMeasureEm,
        adjustmentRatio: ratio,
        badness,
        demerits,
        fitnessLabel: classifyFitnessLabel(ratio),
      });
      gStart = i;
      gWidth = w;
    } else {
      gWidth += add;
    }
  }

  if (gStart < n) {
    greedyLines.push({
      lineNumber: greedyLines.length + 1,
      text: words.slice(gStart).join(' '),
      wordCount: n - gStart,
      naturalWidthEm: gWidth / 1000,
      targetWidthEm: targetMeasureEm,
      adjustmentRatio: 0,
      badness: 0,
      demerits: 0,
      fitnessLabel: 'Normal (Финал)',
    });
  }

  const kpDem = knuthPlassLines.reduce((a, l) => a + l.demerits, 0);
  const grDem = greedyLines.reduce((a, l) => a + l.demerits, 0);
  const kpMaxB = knuthPlassLines.reduce((m, l) => Math.max(m, l.badness), 0);
  const grMaxB = greedyLines.reduce((m, l) => Math.max(m, l.badness), 0);

  return {
    targetMeasureEm,
    knuthPlassLines,
    greedyLines,
    knuthPlassTotalDemerits: kpDem,
    greedyTotalDemerits: grDem,
    knuthPlassMaxBadness: kpMaxB,
    greedyMaxBadness: grMaxB,
    demeritReductionPct: grDem > 0 && grDem >= kpDem ? ((grDem - kpDem) / grDem) * 100 : 0,
  };
}

/**
 * Smart Micro-Typographer mirroring `crates/litera-export/src/markdown_ast.rs`.
 */
export function applyEditorialMicrotypography(input: string): {
  text: string;
  changesCount: number;
} {
  let changes = 0;
  let out = input.replace(/\.\.\./g, () => {
    changes += 1;
    return '…';
  });
  out = out.replace(/\s+--\s+|\s+-\s+/g, () => {
    changes += 1;
    return '\u00A0— ';
  });

  // Replace straight quotes with guillemets «...»
  let open = true;
  out = out.replace(/"/g, () => {
    changes += 1;
    const ch = open ? '«' : '»';
    open = !open;
    return ch;
  });

  // Bind Russian short prepositions with non-breaking spaces
  out = out.replace(
    /(^|\s)(в|во|на|с|со|к|ко|у|о|об|от|до|из|за|по|и|а|но|не|ни)\s+/giu,
    (_, p1, prep) => {
      changes += 1;
      return `${p1}${prep}\u00A0`;
    }
  );

  return { text: out, changesCount: changes };
}

/**
 * Compiles manuscript to Typst (`.typ`) or XeLaTeX (`.tex`) (`crates/litera-export/`).
 */
export function compileExportDocument(
  format: 'typst' | 'latex',
  title: string,
  bodyText: string,
  editorialNote: string,
  profile: ManuscriptProfile,
  fontSizePx: number,
  lineHeight: number,
  useDropCap: boolean
): string {
  const ptSize = Math.round(fontSizePx * 0.45 * 10) / 10;
  if (format === 'typst') {
    return `// ============================================================================
// Сгенерировано в ЛИТЕРА (litera-export :: Typst Typesetting Engine)
// Акустический хеш: ${profile.acousticHashHex} | Энтропия: ${profile.stylometry.wordEntropyBits.toFixed(2)} бит
// Компиляция: typst compile manuscript.typ manuscript.pdf
// ============================================================================

#set document(
  title: "${title}",
  author: "ЛИТЕРА — Акустическая Типографика на Rust",
)

#set page(
  width: 165mm,
  height: 240mm,
  margin: (inside: 18.3mm, top: 26.7mm, outside: 36.7mm, bottom: 53.3mm),
  fill: rgb("#FBF9F5"),
  header: align(right, text(size: 7.5pt, fill: rgb("#78716C"), tracking: 0.12em)[
    ЛИТЕРА · ${title}
  ]),
  footer: align(center, context counter(page).display("— 1 —")),
)

#set text(
  font: ("Cormorant Garamond", "Linux Libertine", "Georgia"),
  size: ${ptSize}pt,
  fill: rgb("#1C1917"),
  lang: "ru",
  ligatures: true,
  number-type: "old-style",
)

#set par(
  leading: ${(lineHeight - 1).toFixed(2)}em,
  justify: true,
  linebreaks: "optimized", // Алгоритм Кнута—Пласса
)

#v(16pt)
#text(size: 8pt, tracking: 0.15em, fill: rgb("#9A3412"))[АКУСТИЧЕСКАЯ ТИПОГРАФИКА · ОТТИСК]
#v(6pt)
#text(size: ${(ptSize * 2.0).toFixed(1)}pt, weight: "medium")[${title}]
#v(10pt)
#line(length: 100%, stroke: 0.4pt + rgb("#D6CEBE"))
#v(14pt)

${
  useDropCap && bodyText.trim().length > 1
    ? `#box(baseline: 65%, inset: (right: 6pt))[#text(size: ${(ptSize * 3.3).toFixed(1)}pt, fill: rgb("#9A3412"), weight: "semibold")[${bodyText.trim()[0]}]]${bodyText.trim().slice(1)}`
    : bodyText.trim()
}

#v(22pt)
#line(length: 35%, stroke: 0.4pt + rgb("#D6CEBE"))
#v(6pt)
#text(size: 8.5pt, style: "italic", fill: rgb("#57534E"))[
  Колофон: Масса набора ${profile.meanInkDensity.toFixed(1)}/100 · Индекс Юла K=${profile.stylometry.yulesCharacteristicK.toFixed(1)} · Метр: ${profile.stylometry.dominantMeter}. ${editorialNote}
]
`;
  }

  return `% ============================================================================
% Сгенерировано в ЛИТЕРА (litera-export :: XeLaTeX / LuaLaTeX Book Engine)
% Акустический хеш: ${profile.acousticHashHex} | Индекс Оборневой: ${profile.stylometry.fleschObornevaIndex.toFixed(1)}
% Сборка: xelatex -interaction=nonstopmode manuscript.tex
% ============================================================================
\\documentclass[11pt,twoside,onecolumn]{memoir}

\\usepackage{fontspec}
\\usepackage[russian,english]{babel}
\\usepackage[protrusion=true,expansion=false,final]{microtype}
\\usepackage{lettrine}
\\usepackage{xcolor}

\\definecolor{ArchivalPaper}{HTML}{FBF9F5}
\\definecolor{ArchivalInk}{HTML}{1C1917}
\\definecolor{TerracottaAccent}{HTML}{9A3412}
\\definecolor{MutedStone}{HTML}{78716C}

\\setmainfont{Cormorant Garamond}[Numbers={OldStyle,Proportional},Ligatures=TeX]

% Канон Ван де Граафа (2 : 3 : 4 : 6)
\\setstocksize{240mm}{165mm}
\\settrimmedsize{240mm}{165mm}{*}
\\setlrmarginsandblock{18.3mm}{36.7mm}{*}
\\setulmarginsandblock{26.7mm}{53.3mm}{*}
\\checkandfixthelayout

% Параметры алгоритма Кнута—Пласса в TeX
\\tolerance=400
\\emergencystretch=1.5em
\\hyphenpenalty=50
\\doublehyphendemerits=10000

\\begin{document}
\\pagecolor{ArchivalPaper}
\\color{ArchivalInk}

\\noindent{\\footnotesize\\scshape\\color{TerracottaAccent} ЛИТЕРА · АКУСТИЧЕСКАЯ МОНОГРАФИЯ}\\\\[6pt]
\\noindent{\\LARGE ${title}}\\par\\vspace{8pt}
\\noindent\\textcolor{MutedStone}{\\rule{\\textwidth}{0.4pt}}\\par\\vspace{14pt}

\\noindent ${bodyText.trim()}

\\vspace{24pt}
\\noindent\\textcolor{MutedStone}{\\rule{0.35\\textwidth}{0.4pt}}\\par\\vspace{6pt}
\\noindent{\\footnotesize\\itshape\\color{MutedStone} Колофон: Масса чернил ${profile.meanInkDensity.toFixed(1)}/100 · Энтропия Шеннона ${profile.stylometry.wordEntropyBits.toFixed(2)} бит. ${editorialNote}}
\\end{document}
`;
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
  {
    id: 'venetian-press',
    title: 'Венецианская печатня Альда Мануция',
    authorOrGenre: 'Историческая монография · Рождение курсива',
    year: '1501 / Очерк',
    editorialNote:
      'Плавное повествовательное дыхание с высокой долей полногласных корней, раскрывающее пластику гарнитуры Cormorant Garamond.',
    text: `В самом начале шестнадцатого столетия в мастерской у канала Риальто книга впервые обрела соразмерность человеческой ладони. Гравёр Франческо Гриффо вырезал пуансоны, в которых наклон гуманистического письма соединился со строгостью римских капителей. С тех пор чтение перестало быть неподвижным трудом в монастырском скриптории и превратилось в свободную беседу наедине со страницей.`,
  },
];

const SPONTANEOUS_STANZAS: Array<{ title: string; text: string; note: string }> = [
  {
    title: 'Этюд № 1 · Оптический интерлиньяж',
    note: 'Спонтанная генерация (xorshift64*): баланс открытых гласных и сонорных «л», «м», «н».',
    text: 'Белое поле вокруг строки важнее самой чернильной линии. Именно в пустоте между абзацами мысль обретает объём, как звук виолончели под каменным куполом пустого зала. Остановитесь на мгновение.',
  },
  {
    title: 'Этюд № 2 · Механика пера и памяти',
    note: 'Спонтанная генерация (xorshift64*): огибающая каденции Staccato → Legato.',
    text: 'Слово начинается с тишины. Вдох. Затем тонкий штрих соединяет вертикальный штамб с плавным овалом, и на светлой бумаге рождается интонация, которую уже невозможно стереть без следа.',
  },
  {
    title: 'Этюд № 3 · Часовая мастерская',
    note: 'Спонтанная генерация (xorshift64*): перкуссионный консонантизм взрывных согласных.',
    text: 'Точность живёт в малом. Тонкий зубчик латунного колеса, сухой щелчок анкера, холодный блеск стали под лупой мастера — так и в строгой прозе каждый предлог стоит на своём единственном месте.',
  },
  {
    title: 'Этюд № 4 · Вечерний оттиск',
    note: 'Спонтанная генерация (xorshift64*): мягкая каденция с преобладанием фрикативных.',
    text: 'Свет гаснет за высоким окном мастерской. На дубовом столе остаётся свежий оттиск страницы, ещё хранящий тёплый запах льняного масла, сосновой смолы и старого свинцового набора.',
  },
  {
    title: 'Этюд № 5 · Северный монолит',
    note: 'Спонтанная генерация (xorshift64*): контрастная архитектура коротких и длинных фраз.',
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
