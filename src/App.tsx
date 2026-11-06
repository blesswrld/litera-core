import React, { useState, useMemo, useEffect } from 'react';
import {
  Volume2,
  Square,
  Sparkles,
  Copy,
  Check,
  Play,
  Download,
  Sliders,
  BookOpen,
} from 'lucide-react';
import {
  analyzeManuscript,
  PRESET_MANUSCRIPTS,
  generateSpontaneousStanza,
  PhonemeClass,
  TempoClass,
  WordMetric,
} from './engine/literaEngine';
import { acousticPlayer } from './engine/acousticSynth';
import {
  RUST_SOURCE_FILES,
  runRustTestSuite,
  RustUnitTestResult,
} from './engine/rustSources';

type ActiveSection = 'editor' | 'cadence' | 'specimen' | 'rust';
type LensMode = 'clean' | 'phonetic' | 'tempo';
type ManuscriptFont = 'serif' | 'sans' | 'mono';

const PHONEME_LABELS: Record<PhonemeClass, { name: string; desc: string; note: string }> = {
  Vowel: {
    name: 'Гласные (Вокализм)',
    desc: 'Открытое дыхание и протяжённость слога (а, е, ё, и, о, у, ы, э, ю, я)',
    note: 'Регистр: D4–C5 (открытый консонанс)',
  },
  Sonorant: {
    name: 'Сонорные',
    desc: 'Плавный и носовой резонанс (л, м, н, р, й)',
    note: 'Регистр: A3–G4 (виолончельный тембр)',
  },
  Fricative: {
    name: 'Фрикативные и щелевые',
    desc: 'Воздушная фактура, шелест и придыхание (в, ф, с, з, ш, щ, ж, х)',
    note: 'Регистр: A4–E5 (верхние обертоны)',
  },
  Plosive: {
    name: 'Взрывные и смычные',
    desc: 'Ритмическая чёткость, удар и контур (б, п, д, т, г, к, ц, ч)',
    note: 'Регистр: G3–A4 (камерное пиццикато)',
  },
  Modifier: {
    name: 'Модификаторы',
    desc: 'Смягчение и разделение (ь, ъ)',
    note: 'Пауза / связка',
  },
  NonPhonetic: {
    name: 'Пунктуация',
    desc: 'Синтаксические цезуры и воздух',
    note: 'Акустическая пауза',
  },
};

const TEMPO_LABELS: Record<TempoClass, { title: string; range: string; advice: string }> = {
  Staccato: {
    title: 'Staccato (Короткий удар)',
    range: '1–5 слов',
    advice: 'Создаёт акцент, останавливает взгляд и фиксирует тезис.',
  },
  Moderato: {
    title: 'Moderato (Средний шаг)',
    range: '6–17 слов',
    advice: 'Основа спокойного повествования и ясной аргументации.',
  },
  Legato: {
    title: 'Legato (Широкий период)',
    range: '18+ слов',
    advice: 'Разворачивает панораму мысли, создавая плавную инерцию чтения.',
  },
};

export default function App() {
  const [darkMode, setDarkMode] = useState<boolean>(false);
  const [activeSection, setActiveSection] = useState<ActiveSection>('editor');

  // Manuscript state
  const [selectedPresetId, setSelectedPresetId] = useState<string>(PRESET_MANUSCRIPTS[0].id);
  const [manuscriptTitle, setManuscriptTitle] = useState<string>(PRESET_MANUSCRIPTS[0].title);
  const [editorialNote, setEditorialNote] = useState<string>(PRESET_MANUSCRIPTS[0].editorialNote);
  const [text, setText] = useState<string>(PRESET_MANUSCRIPTS[0].text);
  const [isEditingRaw, setIsEditingRaw] = useState<boolean>(false);

  // Typographic controls
  const [manuscriptFont, setManuscriptFont] = useState<ManuscriptFont>('serif');
  const [fontSizePx, setFontSizePx] = useState<number>(28);
  const [lineHeight, setLineHeight] = useState<number>(1.75);
  const [useDropCap, setUseDropCap] = useState<boolean>(true);
  const [lensMode, setLensMode] = useState<LensMode>('clean');

  // Interactive selection & audio playback
  const [selectedWordId, setSelectedWordId] = useState<number | null>(null);
  const [playingWordId, setPlayingWordId] = useState<number | null>(null);
  const [isPlayingAudio, setIsPlayingAudio] = useState<boolean>(false);

  // Rust source inspector state
  const [activeRustFileIdx, setActiveRustFileIdx] = useState<number>(0);
  const [copiedCode, setCopiedCode] = useState<boolean>(false);
  const [testResults, setTestResults] = useState<RustUnitTestResult[]>(() => runRustTestSuite());

  useEffect(() => {
    const root = document.documentElement;
    if (darkMode) {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
  }, [darkMode]);

  useEffect(() => {
    return () => {
      acousticPlayer.stop();
    };
  }, []);

  const profile = useMemo(() => analyzeManuscript(text), [text]);

  const selectedWord: WordMetric | null = useMemo(() => {
    if (playingWordId !== null && profile.words[playingWordId]) {
      return profile.words[playingWordId];
    }
    if (selectedWordId !== null && profile.words[selectedWordId]) {
      return profile.words[selectedWordId];
    }
    return profile.words[0] || null;
  }, [profile.words, selectedWordId, playingWordId]);

  const handleSelectPreset = (presetId: string) => {
    const found = PRESET_MANUSCRIPTS.find((p) => p.id === presetId);
    if (!found) return;
    acousticPlayer.stop();
    setIsPlayingAudio(false);
    setPlayingWordId(null);
    setSelectedPresetId(found.id);
    setManuscriptTitle(found.title);
    setEditorialNote(found.editorialNote);
    setText(found.text);
    setSelectedWordId(0);
  };

  const handleSpontaneousStanza = () => {
    acousticPlayer.stop();
    setIsPlayingAudio(false);
    setPlayingWordId(null);
    const stanza = generateSpontaneousStanza();
    setSelectedPresetId('spontaneous');
    setManuscriptTitle(stanza.title);
    setEditorialNote(stanza.note);
    setText(stanza.text);
    setSelectedWordId(0);
  };

  const handleToggleAudio = () => {
    if (isPlayingAudio) {
      acousticPlayer.stop();
      setIsPlayingAudio(false);
      setPlayingWordId(null);
    } else {
      setIsPlayingAudio(true);
      acousticPlayer.playManuscript(
        profile.words,
        (wordIdx) => setPlayingWordId(wordIdx),
        () => {
          setIsPlayingAudio(false);
          setPlayingWordId(null);
        }
      );
    }
  };

  const handleWordClick = (word: WordMetric) => {
    setSelectedWordId(word.id);
    acousticPlayer.playManuscript(
      [word],
      () => {},
      () => {}
    );
  };

  const handleCopyRustCode = () => {
    const code = RUST_SOURCE_FILES[activeRustFileIdx].code;
    navigator.clipboard.writeText(code);
    setCopiedCode(true);
    setTimeout(() => setCopiedCode(false), 1800);
  };

  const handleDownloadRustCrate = () => {
    const bundleText = RUST_SOURCE_FILES.map(
      (f) => `// ==========================================\n// FILE: ${f.path}\n// ${f.role}\n// ==========================================\n\n${f.code}\n\n`
    ).join('\n');
    const blob = new Blob([bundleText], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'litera-core-rust-workspace.rs.txt';
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleRunTests = () => {
    setTestResults(runRustTestSuite());
  };

  const fontClass =
    manuscriptFont === 'serif'
      ? 'font-serif-editorial'
      : manuscriptFont === 'mono'
      ? 'font-mono-tabular'
      : 'font-sans';

  const scrollToSection = (section: ActiveSection) => {
    setActiveSection(section);
    const el = document.getElementById(`section-${section}`);
    if (el) {
      el.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  };

  return (
    <div className="min-h-screen bg-[#FBF9F5] dark:bg-[#0A0A0A] text-[#1C1917] dark:text-[#F5F2EB] transition-colors duration-200">
      {/* Top Bar Contract: [Brand title, one line] — [4 nav links] — [1 primary action] */}
      <header className="sticky top-0 z-30 flex items-center justify-between gap-8 px-6 lg:px-12 py-4 bg-[#FBF9F5]/95 dark:bg-[#0A0A0A]/95 backdrop-blur-sm border-b border-[#E6E1D6] dark:border-[#22201E]">
        <a
          href="#top"
          onClick={(e) => {
            e.preventDefault();
            window.scrollTo({ top: 0, behavior: 'smooth' });
          }}
          className="font-serif-editorial text-2xl font-semibold tracking-tight text-[#1C1917] dark:text-[#F5F2EB] whitespace-nowrap shrink-0"
        >
          ЛИТЕРА
        </a>

        <nav className="hidden md:flex items-center gap-8 text-sm font-medium text-[#57534E] dark:text-[#A8A29E]">
          <button
            type="button"
            onClick={() => scrollToSection('editor')}
            className={`hover:text-[#1C1917] dark:hover:text-[#F5F2EB] transition-colors whitespace-nowrap shrink-0 pb-0.5 ${
              activeSection === 'editor'
                ? 'text-[#1C1917] dark:text-[#F5F2EB] border-b border-[#9A3412] dark:border-[#D97706]'
                : ''
            }`}
          >
            Тетрадь
          </button>
          <button
            type="button"
            onClick={() => scrollToSection('cadence')}
            className={`hover:text-[#1C1917] dark:hover:text-[#F5F2EB] transition-colors whitespace-nowrap shrink-0 pb-0.5 ${
              activeSection === 'cadence'
                ? 'text-[#1C1917] dark:text-[#F5F2EB] border-b border-[#9A3412] dark:border-[#D97706]'
                : ''
            }`}
          >
            Партитура ритма
          </button>
          <button
            type="button"
            onClick={() => scrollToSection('specimen')}
            className={`hover:text-[#1C1917] dark:hover:text-[#F5F2EB] transition-colors whitespace-nowrap shrink-0 pb-0.5 ${
              activeSection === 'specimen'
                ? 'text-[#1C1917] dark:text-[#F5F2EB] border-b border-[#9A3412] dark:border-[#D97706]'
                : ''
            }`}
          >
            Оптическая шкала
          </button>
          <button
            type="button"
            onClick={() => scrollToSection('rust')}
            className={`hover:text-[#1C1917] dark:hover:text-[#F5F2EB] transition-colors whitespace-nowrap shrink-0 pb-0.5 ${
              activeSection === 'rust'
                ? 'text-[#1C1917] dark:text-[#F5F2EB] border-b border-[#9A3412] dark:border-[#D97706]'
                : ''
            }`}
          >
            Ядро Rust
          </button>
        </nav>

        <div className="flex items-center shrink-0">
          <button
            type="button"
            onClick={handleToggleAudio}
            className="inline-flex items-center gap-2 px-4 py-2 text-xs font-medium text-[#FBF9F5] dark:text-[#0A0A0A] bg-[#1C1917] dark:bg-[#F5F2EB] hover:bg-[#9A3412] dark:hover:bg-[#D97706] dark:hover:text-[#0A0A0A] rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer"
          >
            {isPlayingAudio ? (
              <>
                <Square className="w-3.5 h-3.5 fill-current" />
                <span>Остановить каденцию</span>
              </>
            ) : (
              <>
                <Volume2 className="w-3.5 h-3.5" />
                <span>Озвучить ритм</span>
              </>
            )}
          </button>
        </div>
      </header>

      {/* Main Container (1440px baseline with generous museum spacing) */}
      <main id="top" className="max-w-[1360px] mx-auto px-6 lg:px-12 py-10 lg:py-16 space-y-20">
        {/* Editorial Hero & Utility Bar */}
        <section className="space-y-8">
          <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-8 pb-8 border-b border-[#E6E1D6] dark:border-[#22201E]">
            <div className="space-y-4 max-w-3xl">
              <div className="flex flex-wrap items-center gap-2 text-xs tracking-widest uppercase text-[#78716C] dark:text-[#A8A29E]">
                <span>Акустическая типографика</span>
                <span aria-hidden="true">·</span>
                <span>Фонетическая каденция</span>
                <span aria-hidden="true">·</span>
                <span>Rust Edition 2021</span>
              </div>
              <h1
                className="font-serif-editorial text-4xl sm:text-5xl lg:text-[3.5rem] font-normal leading-[1.08] tracking-tight text-[#1C1917] dark:text-[#F5F2EB]"
                style={{ textWrap: 'balance' }}
              >
                Музыка предложения и оптическая плотность печатного листа
              </h1>
              <p className="text-base lg:text-lg text-[#57534E] dark:text-[#A8A29E] leading-relaxed max-w-2xl">
                Минималистичная лаборатория для работы со словом. Вычислительное ядро на Rust за один проход{' '}
                <span className="font-mono-tabular text-xs">O(N)</span> измеряет вокализм, чередование коротких и длинных фраз,
                а также относительную массу чернил каждого глифа на бумаге.
              </p>
            </div>

            {/* Workspace Mode & Theme Controls (placed in workspace bar per Top Bar Contract) */}
            <div className="flex flex-wrap items-center gap-3 shrink-0">
              <button
                type="button"
                onClick={handleSpontaneousStanza}
                className="inline-flex items-center gap-2 px-4 py-2 text-xs font-medium border border-[#D6CEBE] dark:border-[#2E2B28] text-[#1C1917] dark:text-[#F5F2EB] hover:border-[#9A3412] dark:hover:border-[#D97706] rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer"
              >
                <Sparkles className="w-3.5 h-3.5 text-[#9A3412] dark:text-[#D97706]" />
                <span>Спонтанный этюд</span>
              </button>

              <div className="flex items-center p-0.5 bg-[#F2EFE9] dark:bg-[#161514] border border-[#E6E1D6] dark:border-[#262422] rounded">
                <button
                  type="button"
                  onClick={() => setDarkMode(false)}
                  className={`px-3 py-1.5 text-xs font-medium rounded-sm transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                    !darkMode
                      ? 'bg-[#FBF9F5] text-[#1C1917] shadow-xs'
                      : 'text-[#A8A29E] hover:text-[#F5F2EB]'
                  }`}
                >
                  Бумага
                </button>
                <button
                  type="button"
                  onClick={() => setDarkMode(true)}
                  className={`px-3 py-1.5 text-xs font-medium rounded-sm transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                    darkMode
                      ? 'bg-[#262422] text-[#F5F2EB] shadow-xs'
                      : 'text-[#57534E] hover:text-[#1C1917]'
                  }`}
                >
                  Тушь
                </button>
              </div>
            </div>
          </div>

          {/* Full-Width 4-Column Metric Strip below Hero */}
          <div className="grid grid-cols-2 lg:grid-cols-4 gap-6 lg:gap-8 pb-8 border-b border-[#E6E1D6] dark:border-[#22201E]">
            <div className="space-y-1">
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                Слов и слогов в отрывке
              </div>
              <div className="font-serif-editorial text-3xl lg:text-4xl font-semibold text-[#1C1917] dark:text-[#F5F2EB] font-mono-tabular">
                {profile.wordCount}{' '}
                <span className="text-lg font-normal text-[#78716C] dark:text-[#A8A29E]">
                  / {profile.syllableCount} сл.
                </span>
              </div>
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                Средняя длина: {(profile.syllableCount / Math.max(1, profile.wordCount)).toFixed(2)} слога на слово
              </div>
            </div>

            <div className="space-y-1">
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                Оптическая плотность (Ink Density)
              </div>
              <div className="font-serif-editorial text-3xl lg:text-4xl font-semibold text-[#1C1917] dark:text-[#F5F2EB] font-mono-tabular">
                {profile.meanInkDensity.toFixed(1)}
                <span className="text-lg font-normal text-[#78716C] dark:text-[#A8A29E]"> / 100</span>
              </div>
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                {profile.meanInkDensity > 60
                  ? 'Плотный, насыщенный набор'
                  : profile.meanInkDensity < 52
                  ? 'Светлая, воздушная полоса'
                  : 'Классический баланс серого поля'}
              </div>
            </div>

            <div className="space-y-1">
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                Индекс динамики ритма
              </div>
              <div className="font-serif-editorial text-3xl lg:text-4xl font-semibold text-[#1C1917] dark:text-[#F5F2EB] font-mono-tabular">
                {(profile.cadenceVariance * 100).toFixed(0)}%
              </div>
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                {profile.cadenceVariance >= 0.35
                  ? 'Выразительный контраст длин фраз'
                  : 'Ровный, размеренный шаг строк'}
              </div>
            </div>

            <div className="space-y-1">
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                Акустический хеш FNV-1a (64-bit)
              </div>
              <div className="font-mono-tabular text-base lg:text-lg font-medium text-[#9A3412] dark:text-[#D97706] pt-1.5 truncate">
                {profile.acousticHashHex}
              </div>
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E] font-mono-tabular">
                Анализ за {profile.executionTimeMicros} мкс · {profile.charCount} симв.
              </div>
            </div>
          </div>
        </section>

        {/* SECTION 01: MANUSCRIPT & LIVE ACOUSTIC SHEET */}
        <section id="section-editor" className="space-y-8 scroll-mt-24">
          <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
            <div className="space-y-1">
              <h2 className="font-serif-editorial text-2xl lg:text-3xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                01. Тетрадь и акустический разбор
              </h2>
              <p className="text-sm text-[#57534E] dark:text-[#A8A29E]">
                Выберите один из архивных текстов или переключитесь в режим свободного письма. Нажмите на любое слово, чтобы услышать его камерный интервал.
              </p>
            </div>

            {/* Preset Switcher + Lens Mode */}
            <div className="flex flex-wrap items-center gap-2">
              {PRESET_MANUSCRIPTS.map((preset) => (
                <button
                  key={preset.id}
                  type="button"
                  onClick={() => handleSelectPreset(preset.id)}
                  className={`px-3 py-1.5 text-xs font-medium rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                    selectedPresetId === preset.id
                      ? 'bg-[#1C1917] text-[#FBF9F5] dark:bg-[#F5F2EB] dark:text-[#0A0A0A]'
                      : 'bg-[#F2EFE9] dark:bg-[#161514] text-[#57534E] dark:text-[#A8A29E] hover:text-[#1C1917] dark:hover:text-[#F5F2EB]'
                  }`}
                >
                  {preset.title.split(' ')[0]} {preset.title.split(' ')[1] || ''}
                </button>
              ))}
            </div>
          </div>

          {/* Asymmetric 2-Column Editorial Canvas (70% Manuscript / 30% Margin Inspector) */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 items-start">
            {/* Left 8 Columns: The Manuscript Sheet */}
            <div className="lg:col-span-8 bg-[#F7F4EE] dark:bg-[#121110] border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 sm:p-10 lg:p-12 space-y-8">
              {/* Sheet Top Controls */}
              <div className="flex flex-wrap items-center justify-between gap-4 pb-6 border-b border-[#E6E1D6] dark:border-[#22201E]">
                <div className="space-y-1">
                  <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                    {manuscriptTitle} · Время чтения ~{profile.readingTimeSeconds} сек.
                  </div>
                </div>

                <div className="flex flex-wrap items-center gap-2">
                  {/* Lens Switcher */}
                  <div className="flex items-center p-0.5 bg-[#EBE6DF] dark:bg-[#1C1A18] rounded">
                    <button
                      type="button"
                      onClick={() => setLensMode('clean')}
                      className={`px-2.5 py-1 text-xs font-medium rounded-xs transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                        lensMode === 'clean'
                          ? 'bg-[#FBF9F5] dark:bg-[#2C2926] text-[#1C1917] dark:text-[#F5F2EB]'
                          : 'text-[#57534E] dark:text-[#A8A29E]'
                      }`}
                    >
                      Чистый лист
                    </button>
                    <button
                      type="button"
                      onClick={() => setLensMode('phonetic')}
                      className={`px-2.5 py-1 text-xs font-medium rounded-xs transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                        lensMode === 'phonetic'
                          ? 'bg-[#FBF9F5] dark:bg-[#2C2926] text-[#1C1917] dark:text-[#F5F2EB]'
                          : 'text-[#57534E] dark:text-[#A8A29E]'
                      }`}
                    >
                      Фонетика
                    </button>
                    <button
                      type="button"
                      onClick={() => setLensMode('tempo')}
                      className={`px-2.5 py-1 text-xs font-medium rounded-xs transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                        lensMode === 'tempo'
                          ? 'bg-[#FBF9F5] dark:bg-[#2C2926] text-[#1C1917] dark:text-[#F5F2EB]'
                          : 'text-[#57534E] dark:text-[#A8A29E]'
                      }`}
                    >
                      Каденция
                    </button>
                  </div>

                  <button
                    type="button"
                    onClick={() => setIsEditingRaw(!isEditingRaw)}
                    className={`px-3 py-1 text-xs font-medium border rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                      isEditingRaw
                        ? 'border-[#9A3412] dark:border-[#D97706] text-[#9A3412] dark:text-[#D97706]'
                        : 'border-[#D6CEBE] dark:border-[#2E2B28] text-[#57534E] dark:text-[#A8A29E] hover:text-[#1C1917] dark:hover:text-[#F5F2EB]'
                    }`}
                  >
                    {isEditingRaw ? 'Готово (Просмотр)' : 'Редактировать текст'}
                  </button>
                </div>
              </div>

              {/* Either Raw Textarea or Interactive Typographic Sheet */}
              {isEditingRaw ? (
                <div className="space-y-3">
                  <label htmlFor="manuscript-textarea" className="block text-xs text-[#78716C] dark:text-[#A8A29E]">
                    Введите или вставьте собственный текст (на русском или английском языке):
                  </label>
                  <textarea
                    id="manuscript-textarea"
                    value={text}
                    onChange={(e) => {
                      setSelectedPresetId('custom');
                      setManuscriptTitle('Пользовательская рукопись');
                      setEditorialNote('Живой расчёт фонетического спектра и оптической плотности в реальном времени.');
                      setText(e.target.value);
                    }}
                    rows={8}
                    placeholder="Напишите несколько предложений..."
                    className={`w-full bg-[#FBF9F5] dark:bg-[#0A0A0A] border border-[#D6CEBE] dark:border-[#2E2B28] rounded p-5 text-[#1C1917] dark:text-[#F5F2EB] focus:outline-none focus:border-[#9A3412] dark:focus:border-[#D97706] ${fontClass}`}
                    style={{
                      fontSize: `${Math.min(fontSizePx, 24)}px`,
                      lineHeight: lineHeight,
                    }}
                  />
                </div>
              ) : profile.words.length === 0 ? (
                <div className="py-12 text-center space-y-3">
                  <p className="text-sm text-[#78716C] dark:text-[#A8A29E]">
                    Лист пуст. Нажмите «Редактировать текст» или выберите один из этюдов выше.
                  </p>
                </div>
              ) : lensMode === 'tempo' ? (
                /* Sentence Cadence View */
                <div
                  className={`max-w-[68ch] text-[#1C1917] dark:text-[#F5F2EB] ${fontClass} ${
                    useDropCap ? 'editorial-dropcap' : ''
                  }`}
                  style={{
                    fontSize: `${fontSizePx}px`,
                    lineHeight: lineHeight,
                  }}
                >
                  {profile.sentences.map((s) => {
                    const underlineClass =
                      s.tempoTag === 'Staccato'
                        ? 'decoration-[#9A3412] dark:decoration-[#D97706] underline decoration-2 underline-offset-4'
                        : s.tempoTag === 'Legato'
                        ? 'decoration-[#1E3A8A] dark:decoration-[#60A5FA] underline decoration-1 underline-offset-4'
                        : 'decoration-[#A8A29E]/50 underline decoration-1 underline-offset-4';
                    return (
                      <span key={s.index} className={`${underlineClass} mr-2 transition-colors`}>
                        {s.rawSlice}{' '}
                      </span>
                    );
                  })}
                </div>
              ) : (
                /* Interactive Word-by-Word Sheet */
                <div
                  className={`max-w-[68ch] text-[#1C1917] dark:text-[#F5F2EB] ${fontClass} ${
                    useDropCap ? 'editorial-dropcap' : ''
                  }`}
                  style={{
                    fontSize: `${fontSizePx}px`,
                    lineHeight: lineHeight,
                  }}
                >
                  {profile.words.map((w) => {
                    const isActive = playingWordId === w.id || selectedWord?.id === w.id;
                    let phoneticStyle = '';
                    if (lensMode === 'phonetic') {
                      if (w.dominantClass === 'Sonorant') {
                        phoneticStyle = 'border-b-2 border-[#9A3412]/70 dark:border-[#D97706]/80';
                      } else if (w.dominantClass === 'Fricative') {
                        phoneticStyle = 'border-b border-[#1E3A8A]/70 dark:border-[#60A5FA]/80';
                      } else if (w.dominantClass === 'Plosive') {
                        phoneticStyle = 'border-b-2 border-dotted border-[#14532D]/80 dark:border-[#34D399]/80';
                      }
                    }

                    return (
                      <React.Fragment key={w.id}>
                        <span
                          onClick={() => handleWordClick(w)}
                          role="button"
                          tabIndex={0}
                          onKeyDown={(e) => {
                            if (e.key === 'Enter' || e.key === ' ') {
                              e.preventDefault();
                              handleWordClick(w);
                            }
                          }}
                          className={`inline cursor-pointer rounded-xs transition-colors duration-150 ${phoneticStyle} ${
                            isActive
                              ? 'bg-[#9A3412]/15 dark:bg-[#D97706]/25 text-[#9A3412] dark:text-[#FBBF24]'
                              : 'hover:bg-[#1C1917]/5 dark:hover:bg-[#F5F2EB]/10'
                          }`}
                        >
                          {w.rawToken}
                        </span>{' '}
                      </React.Fragment>
                    );
                  })}
                </div>
              )}

              {/* Sheet Footer Note & Legend */}
              <div className="pt-6 border-t border-[#E6E1D6] dark:border-[#22201E] flex flex-col sm:flex-row sm:items-center justify-between gap-4 text-xs text-[#78716C] dark:text-[#A8A29E]">
                <p className="italic font-serif-editorial text-sm text-[#57534E] dark:text-[#A8A29E]">
                  Примечание: {editorialNote}
                </p>

                {lensMode === 'phonetic' && (
                  <div className="flex flex-wrap items-center gap-4 shrink-0">
                    <span className="border-b-2 border-[#9A3412] dark:border-[#D97706] pb-0.5">
                      Сонорные
                    </span>
                    <span className="border-b border-[#1E3A8A] dark:border-[#60A5FA] pb-0.5">
                      Щелевые
                    </span>
                    <span className="border-b-2 border-dotted border-[#14532D] dark:border-[#34D399] pb-0.5">
                      Взрывные
                    </span>
                  </div>
                )}

                {lensMode === 'tempo' && (
                  <div className="flex flex-wrap items-center gap-4 shrink-0">
                    <span className="underline decoration-[#9A3412] dark:decoration-[#D97706] decoration-2 underline-offset-4">
                      Staccato (1–5 сл.)
                    </span>
                    <span className="underline decoration-[#A8A29E] decoration-1 underline-offset-4">
                      Moderato (6–17 сл.)
                    </span>
                    <span className="underline decoration-[#1E3A8A] dark:decoration-[#60A5FA] decoration-1 underline-offset-4">
                      Legato (18+ сл.)
                    </span>
                  </div>
                )}
              </div>
            </div>

            {/* Right 4 Columns: Margin Inspector (Word Anatomy & Phonetic Distribution) */}
            <aside className="lg:col-span-4 space-y-8">
              {/* Selected Word Specimen Card */}
              <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-5 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
                <div className="flex items-center justify-between text-xs text-[#78716C] dark:text-[#A8A29E]">
                  <span>Анатомия выбранного слова</span>
                  <span className="font-mono-tabular">
                    {selectedWord ? `#${selectedWord.id + 1} из ${profile.wordCount}` : '—'}
                  </span>
                </div>

                {selectedWord ? (
                  <>
                    <div className="py-3 border-y border-[#E6E1D6] dark:border-[#22201E] flex items-baseline justify-between gap-4">
                      <div className="font-serif-editorial text-4xl font-medium text-[#1C1917] dark:text-[#F5F2EB] tracking-tight break-all">
                        {selectedWord.slice}
                      </div>
                      <button
                        type="button"
                        onClick={() => handleWordClick(selectedWord)}
                        className="text-xs font-medium text-[#9A3412] dark:text-[#D97706] hover:underline whitespace-nowrap shrink-0 cursor-pointer"
                      >
                        Звук слога →
                      </button>
                    </div>

                    <dl className="space-y-3 text-xs">
                      <div className="flex justify-between py-1 border-b border-[#E6E1D6]/60 dark:border-[#22201E]">
                        <dt className="text-[#78716C] dark:text-[#A8A29E]">Слоговая структура</dt>
                        <dd className="font-mono-tabular font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                          {selectedWord.syllables} {selectedWord.syllables === 1 ? 'слог' : selectedWord.syllables < 5 ? 'слога' : 'слогов'} · {selectedWord.charCount} букв
                        </dd>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[#E6E1D6]/60 dark:border-[#22201E]">
                        <dt className="text-[#78716C] dark:text-[#A8A29E]">Доминанта звучания</dt>
                        <dd className="font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                          {PHONEME_LABELS[selectedWord.dominantClass].name}
                        </dd>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[#E6E1D6]/60 dark:border-[#22201E]">
                        <dt className="text-[#78716C] dark:text-[#A8A29E]">Оптическая масса глифов</dt>
                        <dd className="font-mono-tabular font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                          {selectedWord.inkDensity.toFixed(1)} / 100
                        </dd>
                      </div>
                      <div className="flex justify-between py-1">
                        <dt className="text-[#78716C] dark:text-[#A8A29E]">Акустический регистр</dt>
                        <dd className="text-[#57534E] dark:text-[#A8A29E]">
                          {PHONEME_LABELS[selectedWord.dominantClass].note}
                        </dd>
                      </div>
                    </dl>
                  </>
                ) : (
                  <p className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                    Выберите любое слово на листе для разбора его фонетики.
                  </p>
                )}
              </div>

              {/* Phonetic Spectrum Breakdown */}
              <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-5 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
                <div className="space-y-1">
                  <h3 className="font-serif-editorial text-xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                    Фонетический баланс текста
                  </h3>
                  <p className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                    Распределение {profile.letterCount} звучащих графем по акустическим классам
                  </p>
                </div>

                <div className="space-y-4">
                  {[
                    {
                      cls: 'Vowel' as PhonemeClass,
                      ratio: profile.vowelRatio,
                      barColor: 'bg-[#9A3412] dark:bg-[#D97706]',
                    },
                    {
                      cls: 'Sonorant' as PhonemeClass,
                      ratio: profile.sonorantRatio,
                      barColor: 'bg-[#57534E] dark:bg-[#A8A29E]',
                    },
                    {
                      cls: 'Fricative' as PhonemeClass,
                      ratio: profile.fricativeRatio,
                      barColor: 'bg-[#1E3A8A] dark:bg-[#60A5FA]',
                    },
                    {
                      cls: 'Plosive' as PhonemeClass,
                      ratio: profile.plosiveRatio,
                      barColor: 'bg-[#14532D] dark:bg-[#34D399]',
                    },
                  ].map((item) => (
                    <div key={item.cls} className="space-y-1.5">
                      <div className="flex items-baseline justify-between text-xs">
                        <span className="font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                          {PHONEME_LABELS[item.cls].name}
                        </span>
                        <span className="font-mono-tabular text-[#57534E] dark:text-[#A8A29E]">
                          {(item.ratio * 100).toFixed(1)}%
                        </span>
                      </div>
                      <div className="w-full h-1.5 bg-[#EBE6DF] dark:bg-[#1F1D1B] rounded-full overflow-hidden">
                        <div
                          className={`h-full ${item.barColor} transition-all duration-200`}
                          style={{ width: `${Math.min(100, item.ratio * 100)}%` }}
                        />
                      </div>
                      <p className="text-[11px] text-[#78716C] dark:text-[#A8A29E]">
                        {PHONEME_LABELS[item.cls].desc}
                      </p>
                    </div>
                  ))}
                </div>
              </div>
            </aside>
          </div>
        </section>

        {/* SECTION 02: CADENCE STAVE & SENTENCE ARCHITECTURE */}
        <section id="section-cadence" className="space-y-8 pt-8 border-t border-[#E6E1D6] dark:border-[#22201E] scroll-mt-24">
          <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-6">
            <div className="space-y-2 max-w-2xl">
              <h2 className="font-serif-editorial text-2xl lg:text-3xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                02. Партитура каденции и длина периодов
              </h2>
              <p className="text-sm text-[#57534E] dark:text-[#A8A29E]">
                Визуальная диаграмма дыхания текста. Каждая строка отражает одно предложение: его длину в словах, слоговую насыщенность и темповый класс (Staccato, Moderato, Legato).
              </p>
            </div>

            <div className="text-xs text-[#78716C] dark:text-[#A8A29E] font-mono-tabular">
              Предложений: {profile.sentenceCount} · Лексическое разнообразие (TTR): {(profile.lexicalDiversity * 100).toFixed(0)}%
            </div>
          </div>

          {/* Cadence Stave Visual List */}
          <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg divide-y divide-[#E6E1D6] dark:divide-[#22201E] bg-[#FBF9F5] dark:bg-[#0E0D0C]">
            {profile.sentences.length === 0 ? (
              <div className="p-8 text-center text-sm text-[#78716C] dark:text-[#A8A29E]">
                Нет предложений для построения партитуры.
              </div>
            ) : (
              profile.sentences.map((s) => {
                const maxWords = Math.max(...profile.sentences.map((item) => item.wordCount), 20);
                const widthPct = Math.max(8, Math.min(100, (s.wordCount / maxWords) * 100));
                const barAccent =
                  s.tempoTag === 'Staccato'
                    ? 'bg-[#9A3412] dark:bg-[#D97706]'
                    : s.tempoTag === 'Legato'
                    ? 'bg-[#1E3A8A] dark:bg-[#60A5FA]'
                    : 'bg-[#57534E] dark:bg-[#A8A29E]';

                return (
                  <div
                    key={s.index}
                    className="p-5 lg:px-8 grid grid-cols-1 lg:grid-cols-12 gap-4 items-center hover:bg-[#F7F4EE]/60 dark:hover:bg-[#161514]/60 transition-colors"
                  >
                    <div className="lg:col-span-2 flex items-center gap-3 text-xs">
                      <span className="font-mono-tabular text-[#78716C] dark:text-[#A8A29E]">
                        {String(s.index + 1).padStart(2, '0')}.
                      </span>
                      <span className="font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                        { s.tempoTag }
                      </span>
                    </div>

                    <div className="lg:col-span-5 space-y-2">
                      <div className="flex items-center gap-3">
                        <div className="flex-1 h-2 bg-[#EBE6DF] dark:bg-[#1F1D1B] rounded-full overflow-hidden">
                          <div
                            className={`h-full ${barAccent} rounded-full transition-all duration-300`}
                            style={{ width: `${widthPct}%` }}
                          />
                        </div>
                        <span className="font-mono-tabular text-xs text-[#57534E] dark:text-[#A8A29E] w-28 text-right shrink-0">
                          {s.wordCount} сл. · {s.syllableCount} слог.
                        </span>
                      </div>
                    </div>

                    <div className="lg:col-span-5">
                      <p className="font-serif-editorial text-base lg:text-lg text-[#1C1917] dark:text-[#F5F2EB] line-clamp-2">
                        {s.rawSlice}
                      </p>
                    </div>
                  </div>
                );
              })
            )}
          </div>

          {/* 3-Column Tempo Reference Guide */}
          <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
            {(['Staccato', 'Moderato', 'Legato'] as TempoClass[]).map((t) => {
              const count = profile.sentences.filter((s) => s.tempoTag === t).length;
              return (
                <div
                  key={t}
                  className="p-6 border border-[#E6E1D6] dark:border-[#22201E] rounded-lg space-y-2 bg-[#FBF9F5] dark:bg-[#0E0D0C]"
                >
                  <div className="flex items-baseline justify-between">
                    <h3 className="font-serif-editorial text-xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                      {TEMPO_LABELS[t].title}
                    </h3>
                    <span className="font-mono-tabular text-xs text-[#9A3412] dark:text-[#D97706]">
                      {count} предл.
                    </span>
                  </div>
                  <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                    Диапазон: {TEMPO_LABELS[t].range}
                  </div>
                  <p className="text-xs text-[#57534E] dark:text-[#A8A29E] leading-relaxed pt-1">
                    {TEMPO_LABELS[t].advice}
                  </p>
                </div>
              );
            })}
          </div>
        </section>

        {/* SECTION 03: OPTICAL SCALE & TYPOGRAPHIC SPECIMEN */}
        <section id="section-specimen" className="space-y-8 pt-8 border-t border-[#E6E1D6] dark:border-[#22201E] scroll-mt-24">
          <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-6">
            <div className="space-y-2 max-w-2xl">
              <h2 className="font-serif-editorial text-2xl lg:text-3xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                03. Оптическая шкала и шрифтовая проба
              </h2>
              <p className="text-sm text-[#57534E] dark:text-[#A8A29E]">
                Настройте гарнитуру, кегль, интерлиньяж и буквицу для основного листа тетради. Шрифтовая система построена по правилу «2+1»: ренессансная антиква Cormorant Garamond, геометрический гротеск Plus Jakarta Sans и моноширинный JetBrains Mono.
              </p>
            </div>

            <div className="flex items-center gap-2 text-xs text-[#78716C] dark:text-[#A8A29E]">
              <Sliders className="w-3.5 h-3.5" />
              <span>Применяется к разделу «01. Тетрадь»</span>
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            {/* Font Pairings */}
            <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-5 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                Выбор гарнитуры рукописи
              </div>
              <div className="space-y-3">
                {[
                  {
                    id: 'serif' as ManuscriptFont,
                    name: 'Cormorant Garamond',
                    role: 'Книжная антиква с высоким контрастом штрихов',
                    sampleClass: 'font-serif-editorial text-xl',
                  },
                  {
                    id: 'sans' as ManuscriptFont,
                    name: 'Plus Jakarta Sans',
                    role: 'Современный архитектурный нео-гротеск',
                    sampleClass: 'font-sans text-base font-medium',
                  },
                  {
                    id: 'mono' as ManuscriptFont,
                    name: 'JetBrains Mono',
                    role: 'Инженерный моноширинный ритм и табличные цифры',
                    sampleClass: 'font-mono-tabular text-sm',
                  },
                ].map((f) => (
                  <button
                    key={f.id}
                    type="button"
                    onClick={() => setManuscriptFont(f.id)}
                    className={`w-full text-left p-4 rounded border transition-colors cursor-pointer ${
                      manuscriptFont === f.id
                        ? 'border-[#9A3412] dark:border-[#D97706] bg-[#F7F4EE] dark:bg-[#161514]'
                        : 'border-[#E6E1D6] dark:border-[#22201E] hover:border-[#78716C]'
                    }`}
                  >
                    <div className={`${f.sampleClass} text-[#1C1917] dark:text-[#F5F2EB]`}>
                      {f.name} — Аа Бб Вв Гг
                    </div>
                    <div className="text-xs text-[#78716C] dark:text-[#A8A29E] mt-1">
                      {f.role}
                    </div>
                  </button>
                ))}
              </div>
            </div>

            {/* Size, Leading & Dropcap */}
            <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-6 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
              <div className="space-y-3">
                <div className="flex justify-between text-xs">
                  <span className="text-[#78716C] dark:text-[#A8A29E]">Кегль набора (Font Size)</span>
                  <span className="font-mono-tabular font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                    {fontSizePx}px
                  </span>
                </div>
                <div className="grid grid-cols-4 gap-2">
                  {[20, 24, 28, 34].map((sz) => (
                    <button
                      key={sz}
                      type="button"
                      onClick={() => setFontSizePx(sz)}
                      className={`py-2 text-xs font-mono-tabular rounded border transition-colors cursor-pointer ${
                        fontSizePx === sz
                          ? 'bg-[#1C1917] text-[#FBF9F5] dark:bg-[#F5F2EB] dark:text-[#0A0A0A] border-transparent'
                          : 'border-[#E6E1D6] dark:border-[#22201E] text-[#57534E] dark:text-[#A8A29E]'
                      }`}
                    >
                      {sz}px
                    </button>
                  ))}
                </div>
              </div>

              <div className="space-y-3">
                <div className="flex justify-between text-xs">
                  <span className="text-[#78716C] dark:text-[#A8A29E]">Интерлиньяж (Leading)</span>
                  <span className="font-mono-tabular font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                    {lineHeight.toFixed(2)}
                  </span>
                </div>
                <div className="grid grid-cols-3 gap-2">
                  {[
                    { val: 1.55, label: 'Плотный (1.55)' },
                    { val: 1.75, label: 'Книжный (1.75)' },
                    { val: 1.95, label: 'Воздушный (1.95)' },
                  ].map((lh) => (
                    <button
                      key={lh.val}
                      type="button"
                      onClick={() => setLineHeight(lh.val)}
                      className={`py-2 px-2 text-xs rounded border transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                        lineHeight === lh.val
                          ? 'bg-[#1C1917] text-[#FBF9F5] dark:bg-[#F5F2EB] dark:text-[#0A0A0A] border-transparent'
                          : 'border-[#E6E1D6] dark:border-[#22201E] text-[#57534E] dark:text-[#A8A29E]'
                      }`}
                    >
                      {lh.label}
                    </button>
                  ))}
                </div>
              </div>

              <div className="pt-2 border-t border-[#E6E1D6] dark:border-[#22201E] flex items-center justify-between">
                <div className="space-y-0.5">
                  <div className="text-xs font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                    Инициал (Книжная буквица)
                  </div>
                  <div className="text-[11px] text-[#78716C] dark:text-[#A8A29E]">
                    Увеличенная первая литера абзаца в антикве
                  </div>
                </div>
                <button
                  type="button"
                  onClick={() => setUseDropCap(!useDropCap)}
                  className={`px-3 py-1.5 text-xs font-medium rounded border transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                    useDropCap
                      ? 'bg-[#9A3412] text-[#FBF9F5] dark:bg-[#D97706] dark:text-[#0A0A0A] border-transparent'
                      : 'border-[#E6E1D6] dark:border-[#22201E] text-[#57534E] dark:text-[#A8A29E]'
                  }`}
                >
                  {useDropCap ? 'Включена' : 'Выключена'}
                </button>
              </div>
            </div>

            {/* Optical Ink Weight Matrix */}
            <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-4 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
              <div className="space-y-1">
                <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                  Матрица оптической массы глифов (Rust LUT)
                </div>
                <p className="text-xs text-[#57534E] dark:text-[#A8A29E]">
                  Константная таблица <span className="font-mono-tabular">char_ink_density(ch: char) -&gt; u8</span> оценивает площадь штриха каждой литеры:
                </p>
              </div>

              <div className="grid grid-cols-4 gap-2.5 pt-1">
                {[
                  { glyph: 'Щ ш ж м', weight: '84', label: 'Тяжёлые' },
                  { glyph: 'Д б в п', weight: '68', label: 'Плотные' },
                  { glyph: 'А е о р', weight: '54', label: 'Средние' },
                  { glyph: 'С л э ь', weight: '42', label: 'Светлые' },
                ].map((tier) => (
                  <div
                    key={tier.weight}
                    className="p-3 border border-[#E6E1D6] dark:border-[#22201E] rounded text-center space-y-1"
                  >
                    <div className="font-serif-editorial text-lg font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                      {tier.glyph.split(' ')[0]}
                    </div>
                    <div className="font-mono-tabular text-xs text-[#9A3412] dark:text-[#D97706]">
                      {tier.weight}
                    </div>
                    <div className="text-[11px] text-[#78716C] dark:text-[#A8A29E]">
                      {tier.label}
                    </div>
                  </div>
                ))}
              </div>

              <p className="text-xs text-[#78716C] dark:text-[#A8A29E] leading-relaxed pt-2">
                Тексты с обилием широких многоштамбовых букв («ш», «щ», «ж», «м», «ы») требуют чуть большего интерлиньяжа (1.75–1.95), чтобы сохранить воздух на странице.
              </p>
            </div>
          </div>
        </section>

        {/* SECTION 04: RUST ARCHITECTURE & UNIT TEST SUITE */}
        <section id="section-rust" className="space-y-8 pt-8 border-t border-[#E6E1D6] dark:border-[#22201E] scroll-mt-24">
          <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-6">
            <div className="space-y-2 max-w-2xl">
              <h2 className="font-serif-editorial text-2xl lg:text-3xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                04. Вычислительное ядро на Rust (litera-core)
              </h2>
              <p className="text-sm text-[#57534E] dark:text-[#A8A29E]">
                Полный исходный код крейта в директории <span className="font-mono-tabular">/rust-core/</span>. Алгоритм спроектирован без промежуточных куч-аллокаций строк с использованием срезов со временем жизни <span className="font-mono-tabular">&amp;&apos;a str</span> и экспортами для <span className="font-mono-tabular">wasm32-unknown-unknown</span>.
              </p>
            </div>

            <div className="flex flex-wrap items-center gap-3 shrink-0">
              <button
                type="button"
                onClick={handleRunTests}
                className="inline-flex items-center gap-2 px-3.5 py-2 text-xs font-medium border border-[#D6CEBE] dark:border-[#2E2B28] text-[#1C1917] dark:text-[#F5F2EB] hover:border-[#9A3412] dark:hover:border-[#D97706] rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer"
              >
                <Play className="w-3.5 h-3.5 text-[#9A3412] dark:text-[#D97706]" />
                <span>Запустить cargo test</span>
              </button>

              <button
                type="button"
                onClick={handleDownloadRustCrate}
                className="inline-flex items-center gap-2 px-3.5 py-2 text-xs font-medium bg-[#1C1917] text-[#FBF9F5] dark:bg-[#F5F2EB] dark:text-[#0A0A0A] hover:bg-[#9A3412] dark:hover:bg-[#D97706] rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer"
              >
                <Download className="w-3.5 h-3.5" />
                <span>Скачать крейт Rust</span>
              </button>
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
            {/* Code Viewer (8 cols) */}
            <div className="lg:col-span-8 border border-[#E6E1D6] dark:border-[#22201E] rounded-lg overflow-hidden bg-[#F7F4EE] dark:bg-[#121110]">
              <div className="flex flex-wrap items-center justify-between gap-4 px-5 py-3 border-b border-[#E6E1D6] dark:border-[#22201E] bg-[#F2EFE9] dark:bg-[#161514]">
                <div className="flex flex-wrap items-center gap-1.5">
                  {RUST_SOURCE_FILES.map((file, idx) => (
                    <button
                      key={file.filename}
                      type="button"
                      onClick={() => setActiveRustFileIdx(idx)}
                      className={`px-3 py-1 text-xs font-mono-tabular rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                        activeRustFileIdx === idx
                          ? 'bg-[#1C1917] text-[#FBF9F5] dark:bg-[#F5F2EB] dark:text-[#0A0A0A]'
                          : 'text-[#57534E] dark:text-[#A8A29E] hover:text-[#1C1917] dark:hover:text-[#F5F2EB]'
                      }`}
                    >
                      {file.filename}
                    </button>
                  ))}
                </div>

                <button
                  type="button"
                  onClick={handleCopyRustCode}
                  className="inline-flex items-center gap-1.5 text-xs text-[#57534E] dark:text-[#A8A29E] hover:text-[#1C1917] dark:hover:text-[#F5F2EB] whitespace-nowrap shrink-0 cursor-pointer"
                >
                  {copiedCode ? (
                    <>
                      <Check className="w-3.5 h-3.5 text-[#14532D] dark:text-[#34D399]" />
                      <span>Скопировано</span>
                    </>
                  ) : (
                    <>
                      <Copy className="w-3.5 h-3.5" />
                      <span>Копировать модуль</span>
                    </>
                  )}
                </button>
              </div>

              <div className="px-5 py-2.5 border-b border-[#E6E1D6] dark:border-[#22201E] flex flex-wrap items-center justify-between gap-2 text-xs text-[#78716C] dark:text-[#A8A29E]">
                <span>{RUST_SOURCE_FILES[activeRustFileIdx].role}</span>
                <span className="font-mono-tabular">{RUST_SOURCE_FILES[activeRustFileIdx].complexity}</span>
              </div>

              <pre className="p-5 text-xs font-mono-tabular leading-relaxed overflow-x-auto text-[#1C1917] dark:text-[#E7E5E4] max-h-[460px]">
                <code>{RUST_SOURCE_FILES[activeRustFileIdx].code}</code>
              </pre>
            </div>

            {/* Live Unit Tests & Architectural Guarantees (4 cols) */}
            <div className="lg:col-span-4 space-y-6">
              <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-4 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
                <div className="flex items-center justify-between">
                  <h3 className="font-serif-editorial text-xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                    Модульные тесты крейта
                  </h3>
                  <span className="font-mono-tabular text-xs text-[#14532D] dark:text-[#34D399]">
                    {testResults.filter((t) => t.passed).length} / {testResults.length} PASSED
                  </span>
                </div>

                <div className="divide-y divide-[#E6E1D6] dark:divide-[#22201E]">
                  {testResults.map((test) => (
                    <div key={test.name} className="py-3 space-y-1">
                      <div className="flex items-center justify-between gap-2 text-xs">
                        <span className="font-mono-tabular font-medium text-[#1C1917] dark:text-[#F5F2EB] truncate">
                          {test.name}
                        </span>
                        <span className="font-mono-tabular text-[11px] text-[#14532D] dark:text-[#34D399] shrink-0">
                          ok ({test.durationMicros} µs)
                        </span>
                      </div>
                      <p className="font-mono-tabular text-[11px] text-[#78716C] dark:text-[#A8A29E] break-all">
                        {test.assertionDetail}
                      </p>
                    </div>
                  ))}
                </div>
              </div>

              <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-3 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
                <div className="flex items-center gap-2 text-xs font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                  <BookOpen className="w-3.5 h-3.5 text-[#9A3412] dark:text-[#D97706]" />
                  <span>Инженерные гарантии и сложность</span>
                </div>
                <ul className="space-y-2 text-xs text-[#57534E] dark:text-[#A8A29E] leading-relaxed">
                  <li>
                    <strong className="text-[#1C1917] dark:text-[#F5F2EB]">Линейная сложность O(N):</strong> один проход по скалярным значениям UTF-8 (<span className="font-mono-tabular">char_indices()</span>) без регулярных выражений.
                  </li>
                  <li>
                    <strong className="text-[#1C1917] dark:text-[#F5F2EB]">Заимствование без копирования:</strong> структуры <span className="font-mono-tabular">WordMetric&lt;&apos;a&gt;</span> и <span className="font-mono-tabular">SentenceCadence&lt;&apos;a&gt;</span> ссылаются напрямую на исходный буфер.
                  </li>
                  <li>
                    <strong className="text-[#1C1917] dark:text-[#F5F2EB]">Краевые случаи:</strong> безопасная обработка пустых строк, цепочек тире/многоточий, суррогатных пар и смешанного кириллическо-латинского набора без <span className="font-mono-tabular">unwrap()</span> паник.
                  </li>
                </ul>
              </div>
            </div>
          </div>
        </section>
      </main>

      {/* Quiet Editorial Footer */}
      <footer className="mt-20 border-t border-[#E6E1D6] dark:border-[#22201E] py-8 px-6 lg:px-12">
        <div className="max-w-[1360px] mx-auto flex flex-col sm:flex-row items-center justify-between gap-4 text-xs text-[#78716C] dark:text-[#A8A29E]">
          <div className="flex items-center gap-2">
            <span className="font-serif-editorial text-base font-medium text-[#1C1917] dark:text-[#F5F2EB]">
              ЛИТЕРА
            </span>
            <span aria-hidden="true">·</span>
            <span>Студия акустической типографики и ритмического анализа на Rust</span>
          </div>
          <div className="flex items-center gap-4">
            <span>Гарнитуры: Cormorant Garamond · Plus Jakarta Sans · JetBrains Mono</span>
          </div>
        </div>
      </footer>
    </div>
  );
}
