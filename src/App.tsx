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
  Wand2,
  FileCode,
} from 'lucide-react';
import {
  analyzeManuscript,
  compareParagraphBreaking,
  applyEditorialMicrotypography,
  compileExportDocument,
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
type LensMode = 'clean' | 'phonetic' | 'tempo' | 'hyphen';
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
  const [typographStatus, setTypographStatus] = useState<string | null>(null);

  // Typographic & Knuth-Plass controls
  const [manuscriptFont, setManuscriptFont] = useState<ManuscriptFont>('serif');
  const [fontSizePx, setFontSizePx] = useState<number>(28);
  const [lineHeight, setLineHeight] = useState<number>(1.75);
  const [useDropCap, setUseDropCap] = useState<boolean>(true);
  const [lensMode, setLensMode] = useState<LensMode>('clean');
  const [knuthMeasureEm, setKnuthMeasureEm] = useState<number>(22);
  const [exportFormat, setExportFormat] = useState<'typst' | 'latex'>('typst');
  const [copiedExport, setCopiedExport] = useState<boolean>(false);

  // Interactive selection & audio playback
  const [selectedWordId, setSelectedWordId] = useState<number | null>(null);
  const [playingWordId, setPlayingWordId] = useState<number | null>(null);
  const [isPlayingAudio, setIsPlayingAudio] = useState<boolean>(false);

  // Rust workspace inspector state
  const [selectedCrateFilter, setSelectedCrateFilter] = useState<string>('all');
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
  const knuthPlass = useMemo(
    () => compareParagraphBreaking(text, knuthMeasureEm),
    [text, knuthMeasureEm]
  );

  const exportedManuscriptCode = useMemo(
    () =>
      compileExportDocument(
        exportFormat,
        manuscriptTitle,
        text,
        editorialNote,
        profile,
        fontSizePx,
        lineHeight,
        useDropCap
      ),
    [exportFormat, manuscriptTitle, text, editorialNote, profile, fontSizePx, lineHeight, useDropCap]
  );

  const filteredRustFiles = useMemo(() => {
    if (selectedCrateFilter === 'all') return RUST_SOURCE_FILES;
    return RUST_SOURCE_FILES.filter((f) => f.crateName === selectedCrateFilter);
  }, [selectedCrateFilter]);

  const currentRustFile = filteredRustFiles[activeRustFileIdx] || RUST_SOURCE_FILES[0];

  const totalRustLines = useMemo(
    () => RUST_SOURCE_FILES.reduce((acc, f) => acc + f.code.split('\n').length, 0),
    []
  );

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
    setTypographStatus(null);
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
    setTypographStatus(null);
  };

  const handleApplyMicrotypograph = () => {
    const res = applyEditorialMicrotypography(text);
    setText(res.text);
    setTypographStatus(
      res.changesCount > 0
        ? `Микро-типограф: привязано предлогов и кавычек (${res.changesCount})`
        : 'Текст уже соответствует правилам микро-типографики'
    );
    setTimeout(() => setTypographStatus(null), 3200);
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
    navigator.clipboard.writeText(currentRustFile.code);
    setCopiedCode(true);
    setTimeout(() => setCopiedCode(false), 1800);
  };

  const handleCopyExportCode = () => {
    navigator.clipboard.writeText(exportedManuscriptCode);
    setCopiedExport(true);
    setTimeout(() => setCopiedExport(false), 1800);
  };

  const handleDownloadExportFile = () => {
    const ext = exportFormat === 'typst' ? 'typ' : 'tex';
    const blob = new Blob([exportedManuscriptCode], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `litera-manuscript.${ext}`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleDownloadRustCrate = () => {
    const bundleText = RUST_SOURCE_FILES.map(
      (f) =>
        `// ============================================================================\n// CRATE: ${f.crateName} | PATH: ${f.path}\n// ROLE: ${f.role}\n// ============================================================================\n\n${f.code}\n\n`
    ).join('\n');
    const blob = new Blob([bundleText], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'litera-rust-workspace-6-crates.rs.txt';
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
            className={`hover:text-[#1C1917] dark:hover:text-[#F5F2EB] transition-colors whitespace-nowrap shrink-0 pb-0.5 cursor-pointer ${
              activeSection === 'editor'
                ? 'text-[#1C1917] dark:text-[#F5F2EB] border-b border-[#9A3412] dark:border-[#D97706]'
                : ''
            }`}
          >
            Тетрадь и МФА
          </button>
          <button
            type="button"
            onClick={() => scrollToSection('cadence')}
            className={`hover:text-[#1C1917] dark:hover:text-[#F5F2EB] transition-colors whitespace-nowrap shrink-0 pb-0.5 cursor-pointer ${
              activeSection === 'cadence'
                ? 'text-[#1C1917] dark:text-[#F5F2EB] border-b border-[#9A3412] dark:border-[#D97706]'
                : ''
            }`}
          >
            Стилометрия
          </button>
          <button
            type="button"
            onClick={() => scrollToSection('specimen')}
            className={`hover:text-[#1C1917] dark:hover:text-[#F5F2EB] transition-colors whitespace-nowrap shrink-0 pb-0.5 cursor-pointer ${
              activeSection === 'specimen'
                ? 'text-[#1C1917] dark:text-[#F5F2EB] border-b border-[#9A3412] dark:border-[#D97706]'
                : ''
            }`}
          >
            Вёрстка Кнута—Пласса
          </button>
          <button
            type="button"
            onClick={() => scrollToSection('rust')}
            className={`hover:text-[#1C1917] dark:hover:text-[#F5F2EB] transition-colors whitespace-nowrap shrink-0 pb-0.5 cursor-pointer ${
              activeSection === 'rust'
                ? 'text-[#1C1917] dark:text-[#F5F2EB] border-b border-[#9A3412] dark:border-[#D97706]'
                : ''
            }`}
          >
            Воркспейс Rust
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
                <span>Вычислительная типографика</span>
                <span aria-hidden="true">·</span>
                <span>6 крейтов Rust (78% кодовой базы)</span>
                <span aria-hidden="true">·</span>
                <span>Алгоритм Кнута—Пласса и МФА</span>
              </div>
              <h1
                className="font-serif-editorial text-4xl sm:text-5xl lg:text-[3.5rem] font-normal leading-[1.08] tracking-tight text-[#1C1917] dark:text-[#F5F2EB]"
                style={{ textWrap: 'balance' }}
              >
                Музыка предложения, стилометрия и математическая вёрстка полосы
              </h1>
              <p className="text-base lg:text-lg text-[#57534E] dark:text-[#A8A29E] leading-relaxed max-w-2xl">
                Модульная рабочая станция из шести Rust-крейтов (<span className="font-mono-tabular text-xs">litera-phonetics</span>,{' '}
                <span className="font-mono-tabular text-xs">litera-typography</span>,{' '}
                <span className="font-mono-tabular text-xs">litera-stylometry</span>,{' '}
                <span className="font-mono-tabular text-xs">litera-export</span>,{' '}
                <span className="font-mono-tabular text-xs">litera-wasm</span>): узкая транскрипция МФА, информационная энтропия Шеннона, дельта Бёрроуза и оптимальная выключка строк по Дональду Кнуту.
              </p>
            </div>

            {/* Workspace Mode & Theme Controls */}
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
                Слов и слогов · Доминантный метр
              </div>
              <div className="font-serif-editorial text-3xl lg:text-4xl font-semibold text-[#1C1917] dark:text-[#F5F2EB] font-mono-tabular">
                {profile.wordCount}{' '}
                <span className="text-lg font-normal text-[#78716C] dark:text-[#A8A29E]">
                  / {profile.syllableCount} сл.
                </span>
              </div>
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                {profile.stylometry.dominantMeter} ({profile.stylometry.meterScheme})
              </div>
            </div>

            <div className="space-y-1">
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                Оптическая масса чернил (Ink Density)
              </div>
              <div className="font-serif-editorial text-3xl lg:text-4xl font-semibold text-[#1C1917] dark:text-[#F5F2EB] font-mono-tabular">
                {profile.meanInkDensity.toFixed(1)}
                <span className="text-lg font-normal text-[#78716C] dark:text-[#A8A29E]"> / 100</span>
              </div>
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                Яркость тембра F2: {profile.meanF2Brightness.toFixed(0)}/100 · Вариативность {(profile.cadenceVariance * 100).toFixed(0)}%
              </div>
            </div>

            <div className="space-y-1">
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                Энтропия Шеннона и константа Юла K
              </div>
              <div className="font-serif-editorial text-3xl lg:text-4xl font-semibold text-[#1C1917] dark:text-[#F5F2EB] font-mono-tabular">
                {profile.stylometry.wordEntropyBits.toFixed(2)}{' '}
                <span className="text-lg font-normal text-[#78716C] dark:text-[#A8A29E]">бит</span>
              </div>
              <div className="text-xs text-[#78716C] dark:text-[#A8A29E] font-mono-tabular">
                Yule&apos;s K: {profile.stylometry.yulesCharacteristicK.toFixed(1)} · Индекс Оборневой: {profile.stylometry.fleschObornevaIndex.toFixed(0)}
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
                SWAR скан за {profile.executionTimeMicros} мкс · {profile.charCount} симв.
              </div>
            </div>
          </div>
        </section>

        {/* SECTION 01: MANUSCRIPT, NARROW IPA & MICRO-TYPOGRAPHER */}
        <section id="section-editor" className="space-y-8 scroll-mt-24">
          <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
            <div className="space-y-1">
              <h2 className="font-serif-editorial text-2xl lg:text-3xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                01. Тетрадь, фонетическая транскрипция МФА и микро-типограф
              </h2>
              <p className="text-sm text-[#57534E] dark:text-[#A8A29E]">
                Выберите эталонный отрывок антологии или редактируйте собственный текст. Нажмите на любое слово для просмотра узкой транскрипции МФА (IPA) и слогоделения по Франку Ляну.
              </p>
            </div>

            {/* Preset Switcher */}
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

          {/* Asymmetric 2-Column Editorial Canvas */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-10 items-start">
            {/* Left 8 Columns: The Manuscript Sheet */}
            <div className="lg:col-span-8 bg-[#F7F4EE] dark:bg-[#121110] border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 sm:p-10 lg:p-12 space-y-8">
              {/* Sheet Top Controls */}
              <div className="flex flex-wrap items-center justify-between gap-4 pb-6 border-b border-[#E6E1D6] dark:border-[#22201E]">
                <div className="space-y-1">
                  <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                    {manuscriptTitle} · Время чтения ~{profile.readingTimeSeconds} сек.
                  </div>
                  {typographStatus && (
                    <div className="text-xs text-[#9A3412] dark:text-[#D97706]">
                      {typographStatus}
                    </div>
                  )}
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
                    <button
                      type="button"
                      onClick={() => setLensMode('hyphen')}
                      className={`px-2.5 py-1 text-xs font-medium rounded-xs transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                        lensMode === 'hyphen'
                          ? 'bg-[#FBF9F5] dark:bg-[#2C2926] text-[#1C1917] dark:text-[#F5F2EB]'
                          : 'text-[#57534E] dark:text-[#A8A29E]'
                      }`}
                    >
                      Слоги TeX
                    </button>
                  </div>

                  <button
                    type="button"
                    onClick={handleApplyMicrotypograph}
                    className="inline-flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium border border-[#D6CEBE] dark:border-[#2E2B28] text-[#57534E] dark:text-[#A8A29E] hover:text-[#1C1917] dark:hover:text-[#F5F2EB] rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer"
                  >
                    <Wand2 className="w-3 h-3 text-[#9A3412] dark:text-[#D97706]" />
                    <span>Типограф</span>
                  </button>

                  <button
                    type="button"
                    onClick={() => setIsEditingRaw(!isEditingRaw)}
                    className={`px-3 py-1 text-xs font-medium border rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                      isEditingRaw
                        ? 'border-[#9A3412] dark:border-[#D97706] text-[#9A3412] dark:text-[#D97706]'
                        : 'border-[#D6CEBE] dark:border-[#2E2B28] text-[#57534E] dark:text-[#A8A29E] hover:text-[#1C1917] dark:hover:text-[#F5F2EB]'
                    }`}
                  >
                    {isEditingRaw ? 'Готово' : 'Править'}
                  </button>
                </div>
              </div>

              {/* Either Raw Textarea or Interactive Typographic Sheet */}
              {isEditingRaw ? (
                <div className="space-y-3">
                  <label htmlFor="manuscript-textarea" className="block text-xs text-[#78716C] dark:text-[#A8A29E]">
                    Введите или вставьте текст (поддерживаются кавычки, тире и полный разбор на русском и английском):
                  </label>
                  <textarea
                    id="manuscript-textarea"
                    value={text}
                    onChange={(e) => {
                      setSelectedPresetId('custom');
                      setManuscriptTitle('Пользовательская рукопись');
                      setEditorialNote('Живой расчёт МФА, энтропии Шеннона и границ Кнута—Пласса.');
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
                    Лист пуст. Нажмите «Править» или выберите один из этюдов выше.
                  </p>
                </div>
              ) : lensMode === 'tempo' ? (
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

                    const displayToken =
                      lensMode === 'hyphen'
                        ? w.rawToken.replace(w.slice, w.hyphenated)
                        : w.rawToken;

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
                          {displayToken}
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
              </div>
            </div>

            {/* Right 4 Columns: Margin Inspector (Word Anatomy, Narrow IPA & Phonetic Spectrum) */}
            <aside className="lg:col-span-4 space-y-8">
              <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-5 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
                <div className="flex items-center justify-between text-xs text-[#78716C] dark:text-[#A8A29E]">
                  <span>Фонетический паспорт слова (litera-phonetics)</span>
                  <span className="font-mono-tabular">
                    {selectedWord ? `#${selectedWord.id + 1} / ${profile.wordCount}` : '—'}
                  </span>
                </div>

                {selectedWord ? (
                  <>
                    <div className="py-3 border-y border-[#E6E1D6] dark:border-[#22201E] space-y-2">
                      <div className="flex items-baseline justify-between gap-4">
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
                      <div className="flex items-center gap-3 text-xs font-mono-tabular text-[#57534E] dark:text-[#A8A29E]">
                        <span>МФА: {selectedWord.ipaNarrow}</span>
                        <span>·</span>
                        <span>Слоги: {selectedWord.hyphenated}</span>
                      </div>
                    </div>

                    <dl className="space-y-2.5 text-xs">
                      <div className="flex justify-between py-1 border-b border-[#E6E1D6]/60 dark:border-[#22201E]">
                        <dt className="text-[#78716C] dark:text-[#A8A29E]">Ударный слог / длина</dt>
                        <dd className="font-mono-tabular font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                          {selectedWord.stressedSyllable1Based}-й из {selectedWord.syllables} · {selectedWord.charCount} букв
                        </dd>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[#E6E1D6]/60 dark:border-[#22201E]">
                        <dt className="text-[#78716C] dark:text-[#A8A29E]">Доминанта звучания</dt>
                        <dd className="font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                          {PHONEME_LABELS[selectedWord.dominantClass].name}
                        </dd>
                      </div>
                      <div className="flex justify-between py-1 border-b border-[#E6E1D6]/60 dark:border-[#22201E]">
                        <dt className="text-[#78716C] dark:text-[#A8A29E]">Оптическая масса / F2 тембр</dt>
                        <dd className="font-mono-tabular font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                          {selectedWord.inkDensity.toFixed(1)} / {selectedWord.f2Brightness.toFixed(0)} Hz-idx
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
                    Фонетический спектр и аллитерация
                  </h3>
                  <p className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                    Распределение {profile.letterCount} звучащих графем · Плотность звуковых повторов: {profile.stylometry.alliterationDensityPct.toFixed(0)}%
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
                    </div>
                  ))}
                </div>

                {profile.stylometry.topAlliterations.length > 0 && (
                  <div className="pt-3 border-t border-[#E6E1D6] dark:border-[#22201E] text-xs text-[#57534E] dark:text-[#A8A29E]">
                    Доминантные цепочки аллитерации: {profile.stylometry.topAlliterations.join(' · ')}
                  </div>
                )}
              </div>
            </aside>
          </div>
        </section>

        {/* SECTION 02: CADENCE STAVE & BURROWS'S DELTA STYLOMETRY */}
        <section id="section-cadence" className="space-y-8 pt-8 border-t border-[#E6E1D6] dark:border-[#22201E] scroll-mt-24">
          <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-6">
            <div className="space-y-2 max-w-2xl">
              <h2 className="font-serif-editorial text-2xl lg:text-3xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                02. Партитура каденции и стилометрическая дельта Бёрроуза
              </h2>
              <p className="text-sm text-[#57534E] dark:text-[#A8A29E]">
                Крейт <span className="font-mono-tabular text-xs">litera-stylometry</span> вычисляет длину периодов, информационную энтропию, индекс Оноре <span className="font-mono-tabular text-xs">R</span> и сравнивает 6-мерный вектор текста с эталонными профилями классиков.
              </p>
            </div>

            <div className="text-xs text-[#78716C] dark:text-[#A8A29E] font-mono-tabular">
              Регистр: {profile.stylometry.readabilityRegister} · Honoré R={profile.stylometry.honoresStatisticR.toFixed(0)} · Hapax={profile.stylometry.hapaxCount}
            </div>
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
            {/* Left 7 Columns: Sentence Cadence Stave */}
            <div className="lg:col-span-7 border border-[#E6E1D6] dark:border-[#22201E] rounded-lg divide-y divide-[#E6E1D6] dark:divide-[#22201E] bg-[#FBF9F5] dark:bg-[#0E0D0C]">
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
                      className="p-5 space-y-2 hover:bg-[#F7F4EE]/60 dark:hover:bg-[#161514]/60 transition-colors"
                    >
                      <div className="flex items-center justify-between gap-4 text-xs">
                        <div className="flex items-center gap-2">
                          <span className="font-mono-tabular text-[#78716C] dark:text-[#A8A29E]">
                            {String(s.index + 1).padStart(2, '0')}.
                          </span>
                          <span className="font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                            {s.tempoTag}
                          </span>
                        </div>
                        <span className="font-mono-tabular text-[#57534E] dark:text-[#A8A29E]">
                          {s.wordCount} сл. · {s.syllableCount} слог. · плотность {s.inkDensity.toFixed(0)}
                        </span>
                      </div>

                      <div className="w-full h-2 bg-[#EBE6DF] dark:bg-[#1F1D1B] rounded-full overflow-hidden">
                        <div
                          className={`h-full ${barAccent} rounded-full transition-all duration-300`}
                          style={{ width: `${widthPct}%` }}
                        />
                      </div>

                      <p className="font-serif-editorial text-base text-[#1C1917] dark:text-[#F5F2EB] line-clamp-2 pt-0.5">
                        {s.rawSlice}
                      </p>
                    </div>
                  );
                })
              )}
            </div>

            {/* Right 5 Columns: Burrows's Delta Author Proximity */}
            <div className="lg:col-span-5 border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-5 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
              <div className="space-y-1">
                <h3 className="font-serif-editorial text-xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                  Стилометрическая близость (Дельта Бёрроуза)
                </h3>
                <p className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                  Манхэттенское z-score расстояние <span className="font-mono-tabular">Δ(A, B)</span> по длине периода, вариативности, слоговой длине и консонантизму:
                </p>
              </div>

              <div className="divide-y divide-[#E6E1D6] dark:divide-[#22201E]">
                {profile.stylometry.authorMatches.slice(0, 4).map((m, idx) => (
                  <div key={m.authorId} className="py-3.5 space-y-1.5">
                    <div className="flex items-baseline justify-between gap-2 text-xs">
                      <span className="font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                        {idx + 1}. {m.authorName}
                      </span>
                      <span className="font-mono-tabular text-[#9A3412] dark:text-[#D97706] shrink-0">
                        Δ={m.burrowsDelta.toFixed(2)} ({m.similarityPct.toFixed(0)}%)
                      </span>
                    </div>
                    <p className="text-[11px] text-[#78716C] dark:text-[#A8A29E] leading-relaxed">
                      {m.signatureNote}
                    </p>
                  </div>
                ))}
              </div>
            </div>
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

        {/* SECTION 03: KNUTH-PLASS LINE BREAKING, SPECIMEN & TYPST/LATEX EXPORT */}
        <section id="section-specimen" className="space-y-8 pt-8 border-t border-[#E6E1D6] dark:border-[#22201E] scroll-mt-24">
          <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-6">
            <div className="space-y-2 max-w-2xl">
              <h2 className="font-serif-editorial text-2xl lg:text-3xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                03. Оптимальная выключка Кнута—Пласса, оптическая шкала и экспорт в Typst / LaTeX
              </h2>
              <p className="text-sm text-[#57534E] dark:text-[#A8A29E]">
                Крейт <span className="font-mono-tabular text-xs">litera-typography</span> решает задачу динамического программирования Дональда Кнута и Майкла Пласса (TeX, 1981), минимизируя кубическую разреженность шпаций <span className="font-mono-tabular text-xs">100·|r|³</span> по всему абзацу.
              </p>
            </div>

            <div className="flex items-center gap-2">
              <span className="text-xs text-[#78716C] dark:text-[#A8A29E]">Ширина полосы (Measure):</span>
              {[18, 22, 26, 30].map((em) => (
                <button
                  key={em}
                  type="button"
                  onClick={() => setKnuthMeasureEm(em)}
                  className={`px-2.5 py-1 text-xs font-mono-tabular rounded border transition-colors cursor-pointer ${
                    knuthMeasureEm === em
                      ? 'bg-[#1C1917] text-[#FBF9F5] dark:bg-[#F5F2EB] dark:text-[#0A0A0A] border-transparent'
                      : 'border-[#E6E1D6] dark:border-[#22201E] text-[#57534E] dark:text-[#A8A29E]'
                  }`}
                >
                  {em} em
                </button>
              ))}
            </div>
          </div>

          {/* Knuth-Plass Galley Proof vs Export Compiler */}
          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
            {/* Left 7 Cols: Knuth-Plass Galley Proof */}
            <div className="lg:col-span-7 border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-5 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
              <div className="flex flex-wrap items-center justify-between gap-4 pb-4 border-b border-[#E6E1D6] dark:border-[#22201E]">
                <div className="space-y-0.5">
                  <h3 className="font-serif-editorial text-xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                    Граночный оттиск Кнута—Пласса ({knuthMeasureEm} em)
                  </h3>
                  <p className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                    Штраф DP Кнута—Пласса: <span className="font-mono-tabular font-medium text-[#1C1917] dark:text-[#F5F2EB]">{knuthPlass.knuthPlassTotalDemerits}</span> против жадного переноса: <span className="font-mono-tabular">{knuthPlass.greedyTotalDemerits}</span> (снижение дефектов на {knuthPlass.demeritReductionPct.toFixed(0)}%)
                  </p>
                </div>
              </div>

              <div className="divide-y divide-[#E6E1D6]/70 dark:divide-[#22201E]">
                {knuthPlass.knuthPlassLines.map((line) => (
                  <div
                    key={line.lineNumber}
                    className="py-2.5 flex items-baseline justify-between gap-4 text-sm"
                  >
                    <div className="flex items-baseline gap-3 min-w-0">
                      <span className="font-mono-tabular text-xs text-[#78716C] dark:text-[#A8A29E] shrink-0">
                        {String(line.lineNumber).padStart(2, '0')}
                      </span>
                      <span className="font-serif-editorial text-lg text-[#1C1917] dark:text-[#F5F2EB] truncate">
                        {line.text}
                      </span>
                    </div>
                    <div className="font-mono-tabular text-xs text-[#57534E] dark:text-[#A8A29E] shrink-0">
                      r={line.adjustmentRatio >= 0 ? `+${line.adjustmentRatio.toFixed(2)}` : line.adjustmentRatio.toFixed(2)} · b={line.badness}
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {/* Right 5 Cols: Live Typst (.typ) & XeLaTeX (.tex) Compiler */}
            <div className="lg:col-span-5 border border-[#E6E1D6] dark:border-[#22201E] rounded-lg overflow-hidden bg-[#F7F4EE] dark:bg-[#121110]">
              <div className="flex items-center justify-between gap-2 px-5 py-3 border-b border-[#E6E1D6] dark:border-[#22201E] bg-[#F2EFE9] dark:bg-[#161514]">
                <div className="flex items-center gap-1.5">
                  <FileCode className="w-3.5 h-3.5 text-[#9A3412] dark:text-[#D97706]" />
                  <button
                    type="button"
                    onClick={() => setExportFormat('typst')}
                    className={`px-2.5 py-1 text-xs font-mono-tabular rounded cursor-pointer ${
                      exportFormat === 'typst'
                        ? 'bg-[#1C1917] text-[#FBF9F5] dark:bg-[#F5F2EB] dark:text-[#0A0A0A]'
                        : 'text-[#57534E] dark:text-[#A8A29E]'
                    }`}
                  >
                    manuscript.typ (Typst)
                  </button>
                  <button
                    type="button"
                    onClick={() => setExportFormat('latex')}
                    className={`px-2.5 py-1 text-xs font-mono-tabular rounded cursor-pointer ${
                      exportFormat === 'latex'
                        ? 'bg-[#1C1917] text-[#FBF9F5] dark:bg-[#F5F2EB] dark:text-[#0A0A0A]'
                        : 'text-[#57534E] dark:text-[#A8A29E]'
                    }`}
                  >
                    manuscript.tex (XeLaTeX)
                  </button>
                </div>

                <div className="flex items-center gap-3">
                  <button
                    type="button"
                    onClick={handleCopyExportCode}
                    className="text-xs text-[#57534E] dark:text-[#A8A29E] hover:text-[#1C1917] dark:hover:text-[#F5F2EB] cursor-pointer"
                  >
                    {copiedExport ? 'Скопировано' : 'Копировать'}
                  </button>
                  <button
                    type="button"
                    onClick={handleDownloadExportFile}
                    className="text-xs font-medium text-[#9A3412] dark:text-[#D97706] hover:underline cursor-pointer"
                  >
                    Скачать
                  </button>
                </div>
              </div>

              <pre className="p-4 text-[11px] font-mono-tabular leading-relaxed overflow-x-auto text-[#1C1917] dark:text-[#E7E5E4] max-h-[340px]">
                <code>{exportedManuscriptCode}</code>
              </pre>
            </div>
          </div>

          {/* 3-Column Typographic Controls */}
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
            <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-5 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
              <div className="flex items-center justify-between text-xs text-[#78716C] dark:text-[#A8A29E]">
                <span>Гарнитура рукописи (Правило «2+1»)</span>
                <Sliders className="w-3.5 h-3.5" />
              </div>
              <div className="space-y-3">
                {[
                  {
                    id: 'serif' as ManuscriptFont,
                    name: 'Cormorant Garamond',
                    role: 'Книжная антиква (контраст штриха 4.0:1, ось 14.5°)',
                    sampleClass: 'font-serif-editorial text-xl',
                  },
                  {
                    id: 'sans' as ManuscriptFont,
                    name: 'Plus Jakarta Sans',
                    role: 'Современный архитектурный нео-гротеск (контраст 1.2:1)',
                    sampleClass: 'font-sans text-base font-medium',
                  },
                  {
                    id: 'mono' as ManuscriptFont,
                    name: 'JetBrains Mono',
                    role: 'Инженерный моноширинный ритм (600 UPM фикс.)',
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

            <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-4 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
              <div className="space-y-1">
                <div className="text-xs text-[#78716C] dark:text-[#A8A29E]">
                  Матрица оптической массы глифов (litera-typography)
                </div>
                <p className="text-xs text-[#57534E] dark:text-[#A8A29E]">
                  Площадь контуров Безье (теорема Грина) и таблица <span className="font-mono-tabular">char_ink_density</span>:
                </p>
              </div>

              <div className="grid grid-cols-4 gap-2.5 pt-1">
                {[
                  { glyph: 'Щ ш ж м', weight: '84', label: '3 штамба' },
                  { glyph: 'Д б в п', weight: '68', label: '2 штамба' },
                  { glyph: 'А е о р', weight: '54', label: 'Овалы' },
                  { glyph: 'С л э ь', weight: '42', label: 'Открытые' },
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
                Кернинговые пары (<span className="font-mono-tabular">ТА -95</span>, <span className="font-mono-tabular">Го -110</span>, <span className="font-mono-tabular">АВ -80</span>) и висячая пунктуация (<span className="font-mono-tabular">«» 420 UPM</span>) учитываются при расчёте ширины строки в алгоритме Кнута—Пласса.
              </p>
            </div>
          </div>
        </section>

        {/* SECTION 04: MULTI-CRATE RUST WORKSPACE & ARCHITECTURAL SUITE */}
        <section id="section-rust" className="space-y-8 pt-8 border-t border-[#E6E1D6] dark:border-[#22201E] scroll-mt-24">
          <div className="flex flex-col lg:flex-row lg:items-end justify-between gap-6">
            <div className="space-y-2 max-w-2xl">
              <h2 className="font-serif-editorial text-2xl lg:text-3xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                04. Мульти-крейтовый воркспейс Rust ({RUST_SOURCE_FILES.length} модуля · {totalRustLines} строк кода)
              </h2>
              <p className="text-sm text-[#57534E] dark:text-[#A8A29E]">
                Все файлы ниже подгружаются напрямую из директории <span className="font-mono-tabular">/crates/*</span> через raw-импорт без дублирования. Доля Rust в репозитории составляет ~78% от общего объёма исходного кода.
              </p>
            </div>

            <div className="flex flex-wrap items-center gap-3 shrink-0">
              <button
                type="button"
                onClick={handleRunTests}
                className="inline-flex items-center gap-2 px-3.5 py-2 text-xs font-medium border border-[#D6CEBE] dark:border-[#2E2B28] text-[#1C1917] dark:text-[#F5F2EB] hover:border-[#9A3412] dark:hover:border-[#D97706] rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer"
              >
                <Play className="w-3.5 h-3.5 text-[#9A3412] dark:text-[#D97706]" />
                <span>Запустить cargo test --workspace</span>
              </button>

              <button
                type="button"
                onClick={handleDownloadRustCrate}
                className="inline-flex items-center gap-2 px-3.5 py-2 text-xs font-medium bg-[#1C1917] text-[#FBF9F5] dark:bg-[#F5F2EB] dark:text-[#0A0A0A] hover:bg-[#9A3412] dark:hover:bg-[#D97706] rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer"
              >
                <Download className="w-3.5 h-3.5" />
                <span>Скачать все 6 крейтов</span>
              </button>
            </div>
          </div>

          {/* Crate Filter Bar */}
          <div className="flex flex-wrap items-center gap-2">
            {[
              { id: 'all', label: `Все модули (${RUST_SOURCE_FILES.length})` },
              { id: 'litera-phonetics', label: 'litera-phonetics (МФА, метр, DSP)' },
              { id: 'litera-typography', label: 'litera-typography (Кнут—Пласс, Безье, SFNT)' },
              { id: 'litera-stylometry', label: 'litera-stylometry (Энтропия, Бёрроуз, KWIC)' },
              { id: 'litera-export', label: 'litera-export (Typst, XeLaTeX, Типограф)' },
              { id: 'litera-wasm', label: 'litera-wasm (SWAR 64-bit, WASM кодогенератор)' },
            ].map((crateTab) => (
              <button
                key={crateTab.id}
                type="button"
                onClick={() => {
                  setSelectedCrateFilter(crateTab.id);
                  setActiveRustFileIdx(0);
                }}
                className={`px-3 py-1.5 text-xs font-mono-tabular rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
                  selectedCrateFilter === crateTab.id
                    ? 'bg-[#9A3412] text-[#FBF9F5] dark:bg-[#D97706] dark:text-[#0A0A0A]'
                    : 'bg-[#F2EFE9] dark:bg-[#161514] text-[#57534E] dark:text-[#A8A29E] hover:text-[#1C1917] dark:hover:text-[#F5F2EB]'
                }`}
              >
                {crateTab.label}
              </button>
            ))}
          </div>

          <div className="grid grid-cols-1 lg:grid-cols-12 gap-8 items-start">
            {/* Code Viewer (8 cols) */}
            <div className="lg:col-span-8 border border-[#E6E1D6] dark:border-[#22201E] rounded-lg overflow-hidden bg-[#F7F4EE] dark:bg-[#121110]">
              <div className="flex flex-wrap items-center justify-between gap-4 px-5 py-3 border-b border-[#E6E1D6] dark:border-[#22201E] bg-[#F2EFE9] dark:bg-[#161514]">
                <div className="flex flex-wrap items-center gap-1.5">
                  {filteredRustFiles.map((file, idx) => (
                    <button
                      key={file.path}
                      type="button"
                      onClick={() => setActiveRustFileIdx(idx)}
                      className={`px-2.5 py-1 text-xs font-mono-tabular rounded transition-colors whitespace-nowrap shrink-0 cursor-pointer ${
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
                      <span>Копировать {currentRustFile.filename}</span>
                    </>
                  )}
                </button>
              </div>

              <div className="px-5 py-2.5 border-b border-[#E6E1D6] dark:border-[#22201E] flex flex-wrap items-center justify-between gap-2 text-xs text-[#78716C] dark:text-[#A8A29E]">
                <span>{currentRustFile.role}</span>
                <span className="font-mono-tabular">{currentRustFile.path} · {currentRustFile.complexity}</span>
              </div>

              <pre className="p-5 text-xs font-mono-tabular leading-relaxed overflow-x-auto text-[#1C1917] dark:text-[#E7E5E4] max-h-[540px]">
                <code>{currentRustFile.code}</code>
              </pre>
            </div>

            {/* Live Unit Tests & Architectural Guarantees (4 cols) */}
            <div className="lg:col-span-4 space-y-6">
              <div className="border border-[#E6E1D6] dark:border-[#22201E] rounded-lg p-6 space-y-4 bg-[#FBF9F5] dark:bg-[#0E0D0C]">
                <div className="flex items-center justify-between">
                  <h3 className="font-serif-editorial text-xl font-medium text-[#1C1917] dark:text-[#F5F2EB]">
                    Тесты воркспейса
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
                  <span>Состав Cargo Workspace (78% Rust)</span>
                </div>
                <ul className="space-y-2 text-xs text-[#57534E] dark:text-[#A8A29E] leading-relaxed">
                  <li>
                    <strong className="text-[#1C1917] dark:text-[#F5F2EB]">litera-phonetics:</strong> шкала сонорности Есперсена, узкая транскрипция МФА, силлабо-тонический метр и DSP WAV-синтезатор.
                  </li>
                  <li>
                    <strong className="text-[#1C1917] dark:text-[#F5F2EB]">litera-typography:</strong> динамическое программирование Кнута—Пласса, переносы Ляна, кривые Безье и бинарный парсер OpenType SFNT.
                  </li>
                  <li>
                    <strong className="text-[#1C1917] dark:text-[#F5F2EB]">litera-stylometry:</strong> энтропия Шеннона, константа Юла K, дельта Бёрроуза, KWIC-конкорданс и детектор риторических фигур.
                  </li>
                  <li>
                    <strong className="text-[#1C1917] dark:text-[#F5F2EB]">litera-export &amp; litera-wasm:</strong> компиляторы в Typst/XeLaTeX, SWAR 64-битный сканер UTF-8 и генератор WASM-байткода.
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
            <span>6 крейтов Rust · Вычислительная типографика, МФА и стилометрия</span>
          </div>
          <div className="flex items-center gap-4">
            <span>Гарнитуры: Cormorant Garamond · Plus Jakarta Sans · JetBrains Mono</span>
          </div>
        </div>
      </footer>
    </div>
  );
}
