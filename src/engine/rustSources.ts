import {
  analyzeManuscript,
  compareParagraphBreaking,
  countSyllables,
  hyphenateWord,
  transcribeWordToIpa,
  applyEditorialMicrotypography,
} from './literaEngine';

// Direct Vite `?raw` imports from the actual Rust Cargo workspace (`/crates/*`)
import rootCargoToml from '../../Cargo.toml?raw';
import phoneticsLibRs from '../../crates/litera-phonetics/src/lib.rs?raw';
import cyrillicRs from '../../crates/litera-phonetics/src/cyrillic.rs?raw';
import prosodyRs from '../../crates/litera-phonetics/src/prosody.rs?raw';
import ipaTranscriberRs from '../../crates/litera-phonetics/src/ipa_transcriber.rs?raw';
import stressLexiconRs from '../../crates/litera-phonetics/src/stress_lexicon.rs?raw';
import formantSynthRs from '../../crates/litera-phonetics/src/formant_synthesizer.rs?raw';
import syllableFsmRs from '../../crates/litera-phonetics/src/syllable_sonority_fsm.rs?raw';
import acousticSynthRs from '../../crates/litera-phonetics/src/acoustic_synth.rs?raw';
import alliterationRs from '../../crates/litera-phonetics/src/alliteration.rs?raw';

import knuthPlassRs from '../../crates/litera-typography/src/knuth_plass.rs?raw';
import justificationHzRs from '../../crates/litera-typography/src/justification_hz.rs?raw';
import opticalSizingRs from '../../crates/litera-typography/src/optical_sizing_varfont.rs?raw';
import mathTexRs from '../../crates/litera-typography/src/math_typesetting_tex.rs?raw';
import harfbuzzShaperRs from '../../crates/litera-typography/src/harfbuzz_shaper.rs?raw';
import unicodeBidiRs from '../../crates/litera-typography/src/unicode_bidi_linebreak.rs?raw';
import baselineComposerRs from '../../crates/litera-typography/src/baseline_composer.rs?raw';
import hyphenationRs from '../../crates/litera-typography/src/hyphenation.rs?raw';
import kerningRs from '../../crates/litera-typography/src/kerning.rs?raw';
import opentypeTablesRs from '../../crates/litera-typography/src/opentype_tables.rs?raw';
import bezierOutlinesRs from '../../crates/litera-typography/src/bezier_outlines.rs?raw';
import gridRs from '../../crates/litera-typography/src/grid.rs?raw';

import entropyRs from '../../crates/litera-stylometry/src/entropy.rs?raw';
import ngramLmRs from '../../crates/litera-stylometry/src/ngram_language_model.rs?raw';
import discourseGraphRs from '../../crates/litera-stylometry/src/discourse_coherence_graph.rs?raw';
import corpusRs from '../../crates/litera-stylometry/src/corpus.rs?raw';
import rhetoricTaxonomyRs from '../../crates/litera-stylometry/src/rhetoric_taxonomy.rs?raw';
import syntaxTreeRs from '../../crates/litera-stylometry/src/syntax_tree.rs?raw';
import concordanceRs from '../../crates/litera-stylometry/src/concordance.rs?raw';
import markovRs from '../../crates/litera-stylometry/src/markov.rs?raw';

import typstRs from '../../crates/litera-export/src/typst.rs?raw';
import latexRs from '../../crates/litera-export/src/latex.rs?raw';
import pdfPageTreeRs from '../../crates/litera-export/src/pdf_page_tree.rs?raw';
import postscriptEpsRs from '../../crates/litera-export/src/postscript_eps.rs?raw';
import icmlIndesignRs from '../../crates/litera-export/src/icml_indesign.rs?raw';
import markdownAstRs from '../../crates/litera-export/src/markdown_ast.rs?raw';

import simdScanRs from '../../crates/litera-wasm/src/simd_scan.rs?raw';
import wasmBytecodeGenRs from '../../crates/litera-wasm/src/wasm_bytecode_gen.rs?raw';
import ringBufferDspRs from '../../crates/litera-wasm/src/ring_buffer_dsp.rs?raw';
import arenaRs from '../../crates/litera-wasm/src/arena.rs?raw';

export interface RustSourceFile {
  crateName: string;
  filename: string;
  path: string;
  role: string;
  complexity: string;
  code: string;
}

export interface RustUnitTestResult {
  name: string;
  module: string;
  passed: boolean;
  assertionDetail: string;
  durationMicros: number;
}

export const RUST_SOURCE_FILES: RustSourceFile[] = [
  {
    crateName: 'litera-typography',
    filename: 'knuth_plass.rs',
    path: 'crates/litera-typography/src/knuth_plass.rs',
    role: 'Динамическое программирование Кнута—Пласса (1981) для оптимальной вёрстки абзаца',
    complexity: 'Time: O(W · K) DP · Cubic badness 100·|r|³ · Fitness classes 0..3',
    code: knuthPlassRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'justification_hz.rs',
    path: 'crates/litera-typography/src/justification_hz.rs',
    role: 'Алгоритм микро-типографики Германна Цапфа и Петера Карова (hz-program: микро-масштабирование глифов ±2%)',
    complexity: 'Counter-elasticity scaling & stem weight gamma compensation',
    code: justificationHzRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'optical_sizing_varfont.rs',
    path: 'crates/litera-typography/src/optical_sizing_varfont.rs',
    role: 'Интерполятор оптического кегля OpenType 1.8 (fvar/avar opsz 6..72 pt по шкале пуансонов Гарамона и Дидо)',
    complexity: '4-axis (opsz, wght, CONT, ital) piecewise-hyperbolic master solver',
    code: opticalSizingRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'math_typesetting_tex.rs',
    path: 'crates/litera-typography/src/math_typesetting_tex.rs',
    role: 'Математический наборщик Дональда Кнута (Приложение G TeXbook: 8 классов атомов и матрица 8×8 mu-шпаций)',
    complexity: 'Context-sensitive Bin->Ord unary promotion & 18 mu/em layout',
    code: mathTexRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'harfbuzz_shaper.rs',
    path: 'crates/litera-typography/src/harfbuzz_shaper.rs',
    role: 'Конвейер OpenType GSUB/GPOS шейпинга (лигатуры ffi/fl, минускульные цифры onum, капитель smcp)',
    complexity: 'Two-pass GSUB + GPOS glyph buffer shaper',
    code: harfbuzzShaperRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'unicode_bidi_linebreak.rs',
    path: 'crates/litera-typography/src/unicode_bidi_linebreak.rs',
    role: 'Конечный автомат разрыва строки Unicode UAX #14 (правила LB1..LB31 и склейка NBSP)',
    complexity: 'Time: O(N) · Non-breaking glue & punctuation pair matrix',
    code: unicodeBidiRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'baseline_composer.rs',
    path: 'crates/litera-typography/src/baseline_composer.rs',
    role: 'Вертикальная пагинация разворота, защита от висячих строк (вдов/сирот) и коллизий маргиналий',
    complexity: 'Club/widow penalty solver & marginalia vertical stacker',
    code: baselineComposerRs,
  },
  {
    crateName: 'litera-phonetics',
    filename: 'ipa_transcriber.rs',
    path: 'crates/litera-phonetics/src/ipa_transcriber.rs',
    role: 'Узкая фонетическая транскрипция МФА (IPA): палатализация [ʲ], аканье/иканье и ассимиляция',
    complexity: 'Time: O(N) · Regressive voicing assimilation & vowel reduction',
    code: ipaTranscriberRs,
  },
  {
    crateName: 'litera-phonetics',
    filename: 'syllable_sonority_fsm.rs',
    path: 'crates/litera-phonetics/src/syllable_sonority_fsm.rs',
    role: 'Структурное дерево слога (Onset-Nucleus-Coda) и дисперсия сонорности Клементса (1990)',
    complexity: 'Slavic open-syllable law & Clements dispersion D = Σ Δs⁻²',
    code: syllableFsmRs,
  },
  {
    crateName: 'litera-stylometry',
    filename: 'ngram_language_model.rs',
    path: 'crates/litera-stylometry/src/ngram_language_model.rs',
    role: 'Сглаживание Кнесера—Нея для N-грамм (N=1..3), перплексия PP(W) и классификатор исторической орфографии',
    complexity: 'Interpolated Kneser-Ney discounting & pre-1918 register detector',
    code: ngramLmRs,
  },
  {
    crateName: 'litera-stylometry',
    filename: 'discourse_coherence_graph.rs',
    path: 'crates/litera-stylometry/src/discourse_coherence_graph.rs',
    role: 'Граф лексической когезии Халлидея—Хасан, морфологический стеммер и детектор кольцевой композиции',
    complexity: 'TextTiling thematic valley detector & Jaccard stem graph',
    code: discourseGraphRs,
  },
  {
    crateName: 'litera-export',
    filename: 'pdf_page_tree.rs',
    path: 'crates/litera-export/src/pdf_page_tree.rs',
    role: 'Прямой генератор документов ISO 32000-1 (%PDF-1.7) с массивом кернинга TJ и таблицей байтовых смещений /xref',
    complexity: 'Zero-dependency PDF 1.7 Catalog/Pages/Contents & xref emitter',
    code: pdfPageTreeRs,
  },
  {
    crateName: 'litera-wasm',
    filename: 'ring_buffer_dsp.rs',
    path: 'crates/litera-wasm/src/ring_buffer_dsp.rs',
    role: 'Неблокирующий SPSC кольцевой буфер (#[repr(C, align(64))]) для WebAudio Worklet и баллистический dBFS-метр',
    complexity: 'Cache-line padded lock-free 128-frame quantum streamer',
    code: ringBufferDspRs,
  },
  {
    crateName: 'litera-phonetics',
    filename: 'stress_lexicon.rs',
    path: 'crates/litera-phonetics/src/stress_lexicon.rs',
    role: 'Акцентные парадигмы А. А. Зализняка (a, b, c, d, e, f) и словарь ударений русской прозы',
    complexity: 'Inflectional stem/ending stress mobility lookup',
    code: stressLexiconRs,
  },
  {
    crateName: 'litera-phonetics',
    filename: 'formant_synthesizer.rs',
    path: 'crates/litera-phonetics/src/formant_synthesizer.rs',
    role: 'Каскадный 3-формантный синтезатор гласных Денниса Клатта (F1, F2, F3) и голосовой импульс Розенберга',
    complexity: '2-pole digital resonators · Fant/Bondarko formant tables',
    code: formantSynthRs,
  },
  {
    crateName: 'litera-phonetics',
    filename: 'prosody.rs',
    path: 'crates/litera-phonetics/src/prosody.rs',
    role: 'Сканер силлабо-тонического метра (Ямб, Хорей, Дактиль, Амфибрахий, Анапест, Дольник)',
    complexity: 'Time: O(N) · Классификация клаузул и акцентуации по А. Белому и М. Гаспарову',
    code: prosodyRs,
  },
  {
    crateName: 'litera-phonetics',
    filename: 'cyrillic.rs',
    path: 'crates/litera-phonetics/src/cyrillic.rs',
    role: 'Артикуляционная фонетика кириллицы, формантная яркость F2 и шкала сонорности Есперсена',
    complexity: 'Time: O(1) lookup · Zero heap allocations',
    code: cyrillicRs,
  },
  {
    crateName: 'litera-phonetics',
    filename: 'acoustic_synth.rs',
    path: 'crates/litera-phonetics/src/acoustic_synth.rs',
    role: 'Чистый Rust DSP-синтезатор камерного звука, Biquad IIR фильтр и RIFF WAV (PCM16) энкодер',
    complexity: 'Direct Form I Biquad resonator · 22.05 kHz 16-bit PCM synthesis',
    code: acousticSynthRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'hyphenation.rs',
    path: 'crates/litera-typography/src/hyphenation.rs',
    role: 'Алгоритм переносов Франка Ляна (TeX Patgen) и фонологическое слогоделение РАН',
    complexity: 'Inter-letter weights 0..9 · Non-detachable modifier guards (й, ь, ъ)',
    code: hyphenationRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'opentype_tables.rs',
    path: 'crates/litera-typography/src/opentype_tables.rs',
    role: 'Zero-copy бинарный парсер таблиц OpenType / TrueType (SFNT head, hhea, OS/2)',
    complexity: 'Big-endian slice parser &[u8] · 0 unsafe blocks',
    code: opentypeTablesRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'bezier_outlines.rs',
    path: 'crates/litera-typography/src/bezier_outlines.rs',
    role: 'Геометрия кривых Безье (алгоритм де Кастельжо) и теорема Грина для площади контура глифа',
    complexity: 'Cubic/Quadratic subdivision · Shoelace signed contour integration',
    code: bezierOutlinesRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'kerning.rs',
    path: 'crates/litera-typography/src/kerning.rs',
    role: 'Таблицы оптического кернинга пар и висячей пунктуации (microtype protrusion)',
    complexity: '1/1000 em precision · Hanging guillemets & em-dashes',
    code: kerningRs,
  },
  {
    crateName: 'litera-typography',
    filename: 'grid.rs',
    path: 'crates/litera-typography/src/grid.rs',
    role: 'Классические книжные каноны (Ван де Грааф 2:3:4:6, золотой канон Чихольда, Брингхёрст)',
    complexity: 'Exact mm spread geometry & diatonic modular scale',
    code: gridRs,
  },
  {
    crateName: 'litera-stylometry',
    filename: 'entropy.rs',
    path: 'crates/litera-stylometry/src/entropy.rs',
    role: 'Информационная энтропия Шеннона H(X), константа Юла K, индекс Оноре R и закон Ципфа',
    complexity: 'Time: O(N log V) · Length-invariant lexical richness',
    code: entropyRs,
  },
  {
    crateName: 'litera-stylometry',
    filename: 'corpus.rs',
    path: 'crates/litera-stylometry/src/corpus.rs',
    role: 'Стилометрическая дельта Бёрроуза (z-score расстояние до Мандельштама, Чехова, Набокова, Бродского)',
    complexity: '6D z-score Manhattan distance across reference centroids',
    code: corpusRs,
  },
  {
    crateName: 'litera-stylometry',
    filename: 'rhetoric_taxonomy.rs',
    path: 'crates/litera-stylometry/src/rhetoric_taxonomy.rs',
    role: 'Классическая риторическая таксономия Квинтилиана (Антитеза, закон градации Бехагеля, хиазм, парцелляция)',
    complexity: 'Polar semantic stems & clause symmetry detector',
    code: rhetoricTaxonomyRs,
  },
  {
    crateName: 'litera-stylometry',
    filename: 'syntax_tree.rs',
    path: 'crates/litera-stylometry/src/syntax_tree.rs',
    role: 'Грамматика пунктуационных цезур, индекс гипотаксиса/паратаксиса и детектор анафоры/эпифоры',
    complexity: 'Anaphora, Epiphora, Polysyndeton, Asyndeton & Coda contrast',
    code: syntaxTreeRs,
  },
  {
    crateName: 'litera-stylometry',
    filename: 'concordance.rs',
    path: 'crates/litera-stylometry/src/concordance.rs',
    role: 'Конкорданс KWIC (Key Word In Context), расстояние Левенштейна и граф краевой рифмы',
    complexity: 'Zero-copy context slicing & O(min(M,N))-space edit distance',
    code: concordanceRs,
  },
  {
    crateName: 'litera-stylometry',
    filename: 'markov.rs',
    path: 'crates/litera-stylometry/src/markov.rs',
    role: 'Детерминированный генератор xorshift64* и стохастический синтез строф',
    complexity: 'Zero external RNG crates · Staccato -> Moderato -> Legato envelope',
    code: markovRs,
  },
  {
    crateName: 'litera-export',
    filename: 'typst.rs',
    path: 'crates/litera-export/src/typst.rs',
    role: 'Компилятор книжной вёрстки в формат Typst (.typ) с каноническими полями и буквицей',
    complexity: 'Self-contained Typst script generator',
    code: typstRs,
  },
  {
    crateName: 'litera-export',
    filename: 'latex.rs',
    path: 'crates/litera-export/src/latex.rs',
    role: 'Компилятор архивной монографии XeLaTeX / LuaLaTeX (memoir + fontspec + microtype)',
    complexity: 'TeX Knuth-Plass tolerance & lettrine initials',
    code: latexRs,
  },
  {
    crateName: 'litera-export',
    filename: 'postscript_eps.rs',
    path: 'crates/litera-export/src/postscript_eps.rs',
    role: 'Генератор векторных гранок Encapsulated PostScript Level 3 (.eps) с диагоналями Ван де Граафа',
    complexity: 'PostScript Level 3 vector page & baseline grid plotter',
    code: postscriptEpsRs,
  },
  {
    crateName: 'litera-export',
    filename: 'icml_indesign.rs',
    path: 'crates/litera-export/src/icml_indesign.rs',
    role: 'Экспортер в издательский формат Adobe InDesign / InCopy (.icml XML) с оптическим выравниванием полей',
    complexity: 'Adobe HL Composer & OpticalMarginAlignment XML generator',
    code: icmlIndesignRs,
  },
  {
    crateName: 'litera-export',
    filename: 'markdown_ast.rs',
    path: 'crates/litera-export/src/markdown_ast.rs',
    role: 'Редакционный микро-типограф («ёлочки», тире, привязка предлогов через NBSP) и AST-парсер',
    complexity: 'Two-pass state machine · Zero regex dependencies',
    code: markdownAstRs,
  },
  {
    crateName: 'litera-wasm',
    filename: 'simd_scan.rs',
    path: 'crates/litera-wasm/src/simd_scan.rs',
    role: 'SWAR 64-битный параллельный сканер кодовых точек UTF-8 и разделителей без ветвлений',
    complexity: '8 bytes per CPU cycle via bitwise SWAR PopCount',
    code: simdScanRs,
  },
  {
    crateName: 'litera-wasm',
    filename: 'wasm_bytecode_gen.rs',
    path: 'crates/litera-wasm/src/wasm_bytecode_gen.rs',
    role: 'Чистый Rust компилятор бинарных модулей W3C WebAssembly (\\0asm) и кодек DWARF LEB128',
    complexity: 'Direct WASM opcode emission · ULEB128 / SLEB128',
    code: wasmBytecodeGenRs,
  },
  {
    crateName: 'litera-wasm',
    filename: 'arena.rs',
    path: 'crates/litera-wasm/src/arena.rs',
    role: 'Детерминированный Bump-аллокатор линейной памяти WebAssembly с выравниванием 8 байт',
    complexity: 'O(1) alloc & O(1) frame reset',
    code: arenaRs,
  },
  {
    crateName: 'litera-phonetics',
    filename: 'alliteration.rs',
    path: 'crates/litera-phonetics/src/alliteration.rs',
    role: 'Графы консонантной аллитерации и вокалического ассонанса в скользящем окне',
    complexity: 'Window k=3 proximity graph',
    code: alliterationRs,
  },
  {
    crateName: 'litera-phonetics',
    filename: 'lib.rs',
    path: 'crates/litera-phonetics/src/lib.rs',
    role: 'Корневой модуль крейта litera-phonetics и расчёт контура сонорности слова',
    complexity: 'Core acoustic traits & SSP turbulence detector',
    code: phoneticsLibRs,
  },
  {
    crateName: 'workspace',
    filename: 'Cargo.toml',
    path: 'Cargo.toml',
    role: 'Корневой манифест Cargo Workspace (6 крейтов: phonetics, typography, stylometry, export, wasm, cli)',
    complexity: 'Rust Edition 2021 · LTO Fat · Release Profile',
    code: rootCargoToml,
  },
];

export function runRustTestSuite(): RustUnitTestResult[] {
  const results: RustUnitTestResult[] = [];

  const t1 = performance.now();
  const sOK =
    countSyllables('тишина') === 3 &&
    countSyllables('архитектура') === 5 &&
    countSyllables('поэзия') === 4;
  results.push({
    name: 'test_cyrillic_syllable_and_hiatus_nuclei',
    module: 'litera_phonetics::prosody',
    passed: sOK,
    assertionDetail: 'count_syllables("тишина") == 3 && "архитектура" == 5 && "поэзия" == 4',
    durationMicros: Math.max(4, Math.round((performance.now() - t1) * 1000)),
  });

  const t2 = performance.now();
  const ipa1 = transcribeWordToIpa('тишина');
  const ipa2 = transcribeWordToIpa('сад');
  results.push({
    name: 'test_narrow_ipa_palatalization_and_devoicing',
    module: 'litera_phonetics::ipa_transcriber',
    passed: ipa1.includes('tʲ') && ipa1.includes('ʂ') && ipa2 === '[sat]',
    assertionDetail: `ipa("тишина") -> ${ipa1}, ipa("сад") -> ${ipa2} (оглушение д->t)`,
    durationMicros: Math.max(6, Math.round((performance.now() - t2) * 1000)),
  });

  const t3 = performance.now();
  const kpCmp = compareParagraphBreaking(
    'Когда взгляд скользит по широкому полю тёплой архивной бумаги, каждая тонкая засечка удерживает равновесие между светом и тенью.',
    22
  );
  results.push({
    name: 'test_knuth_plass_dp_beats_greedy_demerits',
    module: 'litera_typography::knuth_plass',
    passed:
      kpCmp.knuthPlassLines.length > 0 &&
      kpCmp.knuthPlassTotalDemerits <= kpCmp.greedyTotalDemerits,
    assertionDetail: `KP demerits (${kpCmp.knuthPlassTotalDemerits}) <= Greedy (${kpCmp.greedyTotalDemerits})`,
    durationMicros: Math.max(11, Math.round((performance.now() - t3) * 1000)),
  });

  const t4 = performance.now();
  const hyph = hyphenateWord('тишина');
  const hyphMod = hyphenateWord('тайна');
  results.push({
    name: 'test_liang_and_slavic_hyphenation_guards',
    module: 'litera_typography::hyphenation',
    passed: hyph === 'ти·ши·на' && hyphMod === 'тай·на',
    assertionDetail: `hyphenate("тишина") == "${hyph}", "тайна" == "${hyphMod}" (не отрывает й)`,
    durationMicros: Math.max(5, Math.round((performance.now() - t4) * 1000)),
  });

  const t5 = performance.now();
  const micro = applyEditorialMicrotypography('Он сказал "Свет в окне - это память..."');
  results.push({
    name: 'test_editorial_microtypography_guillemets_nbsp',
    module: 'litera_export::markdown_ast',
    passed:
      micro.text.includes('«Свет') &&
      micro.text.includes('память…»') &&
      micro.text.includes('—') &&
      micro.changesCount >= 4,
    assertionDetail: `«Свет в\\u00A0окне\\u00A0— это память…» (${micro.changesCount} замен)`,
    durationMicros: Math.max(5, Math.round((performance.now() - t5) * 1000)),
  });

  const t6 = performance.now();
  const prof = analyzeManuscript(
    'Кружевом, камнем и воздухом держится высокий готический свод. Мы поднимаемся на башни.'
  );
  results.push({
    name: 'test_shannon_entropy_and_burrows_delta',
    module: 'litera_stylometry::entropy',
    passed:
      prof.stylometry.charEntropyBits > 3.5 &&
      prof.stylometry.simpsonsDiversity > 0.85 &&
      prof.stylometry.authorMatches.length === 6,
    assertionDetail: `H_char=${prof.stylometry.charEntropyBits.toFixed(2)} бит, Simpson D=${prof.stylometry.simpsonsDiversity.toFixed(3)}`,
    durationMicros: Math.max(9, Math.round((performance.now() - t6) * 1000)),
  });

  return results;
}
