import { PhonemeClass, WordMetric } from './literaEngine';

/**
 * Minimalist Web Audio chamber synthesizer that translates word syllables and
 * dominant phoneme classes into warm acoustic harmonic intervals.
 */
class AcousticCadencePlayer {
  private ctx: AudioContext | null = null;
  private isPlaying = false;
  private timers: number[] = [];

  public playManuscript(
    words: WordMetric[],
    onWordStep: (wordIndex: number | null) => void,
    onComplete: () => void
  ): void {
    this.stop();
    if (words.length === 0) {
      onComplete();
      return;
    }

    const AudioCtx = window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext;
    if (!this.ctx) {
      this.ctx = new AudioCtx();
    }
    if (this.ctx.state === 'suspended') {
      this.ctx.resume();
    }

    this.isPlaying = true;

    // D minor / Dorian chamber pentatonic scale in Hz (warm editorial register)
    const scaleByPhoneme: Record<PhonemeClass, number[]> = {
      Vowel: [293.66, 349.23, 440.0, 523.25],     // D4, F4, A4, C5 (open warm intervals)
      Sonorant: [220.0, 261.63, 329.63, 392.0],   // A3, C4, E4, G4 (resonant cello register)
      Fricative: [440.0, 493.88, 587.33, 659.25], // A4, B4, D5, E5 (delicate airy harmonics)
      Plosive: [196.0, 293.66, 392.0, 440.0],     // G3, D4, G4, A4 (percussive chamber pizzicato)
      Modifier: [329.63],
      NonPhonetic: [293.66],
    };

    let elapsedMs = 40;
    const maxWordsToPlay = Math.min(words.length, 65);

    for (let i = 0; i < maxWordsToPlay; i++) {
      const word = words[i];
      const durationMs = Math.max(150, Math.min(480, word.syllables * 115));
      const hasSentenceBreak = /[.!?…]$/.test(word.rawToken);
      const hasClauseBreak = /[,;:—]$/.test(word.rawToken);

      const timerId = window.setTimeout(() => {
        if (!this.isPlaying || !this.ctx) return;
        onWordStep(word.id);
        this.triggerNote(
          scaleByPhoneme[word.dominantClass],
          word.syllables,
          word.dominantClass,
          durationMs / 1000
        );
      }, elapsedMs);

      this.timers.push(timerId);
      elapsedMs += durationMs + (hasSentenceBreak ? 320 : hasClauseBreak ? 140 : 45);
    }

    const endTimer = window.setTimeout(() => {
      this.isPlaying = false;
      onWordStep(null);
      onComplete();
    }, elapsedMs + 200);

    this.timers.push(endTimer);
  }

  private triggerNote(
    frequencies: number[],
    syllables: number,
    phoneme: PhonemeClass,
    durationSec: number
  ): void {
    if (!this.ctx) return;
    const now = this.ctx.currentTime;
    const freq = frequencies[syllables % frequencies.length];

    const osc = this.ctx.createOscillator();
    const subOsc = this.ctx.createOscillator();
    const gain = this.ctx.createGain();
    const filter = this.ctx.createBiquadFilter();

    osc.type = phoneme === 'Plosive' ? 'triangle' : 'sine';
    subOsc.type = 'sine';

    osc.frequency.setValueAtTime(freq, now);
    subOsc.frequency.setValueAtTime(freq * 0.5, now);

    filter.type = 'lowpass';
    filter.frequency.setValueAtTime(phoneme === 'Fricative' ? 1600 : 950, now);

    const attack = phoneme === 'Plosive' ? 0.012 : 0.035;
    const release = Math.max(0.25, durationSec * 1.15);

    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.exponentialRampToValueAtTime(0.09, now + attack);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + attack + release);

    osc.connect(filter);
    subOsc.connect(filter);
    filter.connect(gain);
    gain.connect(this.ctx.destination);

    osc.start(now);
    subOsc.start(now);
    osc.stop(now + attack + release + 0.05);
    subOsc.stop(now + attack + release + 0.05);
  }

  public stop(): void {
    this.isPlaying = false;
    for (const id of this.timers) {
      window.clearTimeout(id);
    }
    this.timers = [];
  }
}

export const acousticPlayer = new AcousticCadencePlayer();
