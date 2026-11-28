export type SoundEffectKind = "pop" | "happy" | "dizzy" | "alert" | "chime";

let audioCtx: AudioContext | null = null;

function getCtx(): AudioContext | null {
  if (typeof window === "undefined") return null;
  const Ctx =
    window.AudioContext ||
    (window as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext;
  if (!Ctx) return null;
  if (!audioCtx) {
    audioCtx = new Ctx();
  }
  if (audioCtx.state === "suspended") {
    void audioCtx.resume().catch(() => {});
  }
  return audioCtx;
}

export function playGraviSound(kind: SoundEffectKind, enabled: boolean, volumePercent: number): void {
  if (!enabled || volumePercent <= 0) return;
  const ctx = getCtx();
  if (!ctx) return;

  const gainLevel = Math.min(0.25, Math.max(0.01, (volumePercent / 100) * 0.18));
  const now = ctx.currentTime;

  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.connect(gain);
  gain.connect(ctx.destination);

  if (kind === "pop") {
    osc.type = "sine";
    osc.frequency.setValueAtTime(520, now);
    osc.frequency.exponentialRampToValueAtTime(760, now + 0.075);
    gain.gain.setValueAtTime(gainLevel, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.09);
    osc.start(now);
    osc.stop(now + 0.095);
  } else if (kind === "happy") {
    osc.type = "triangle";
    osc.frequency.setValueAtTime(587.33, now);
    osc.frequency.setValueAtTime(880, now + 0.07);
    gain.gain.setValueAtTime(gainLevel, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.19);
    osc.start(now);
    osc.stop(now + 0.2);
  } else if (kind === "dizzy") {
    osc.type = "sine";
    osc.frequency.setValueAtTime(620, now);
    osc.frequency.linearRampToValueAtTime(340, now + 0.11);
    osc.frequency.linearRampToValueAtTime(520, now + 0.2);
    osc.frequency.linearRampToValueAtTime(280, now + 0.3);
    gain.gain.setValueAtTime(gainLevel * 0.9, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.32);
    osc.start(now);
    osc.stop(now + 0.33);
  } else if (kind === "alert") {
    osc.type = "sine";
    osc.frequency.setValueAtTime(740, now);
    osc.frequency.setValueAtTime(987.77, now + 0.09);
    gain.gain.setValueAtTime(gainLevel, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.25);
    osc.start(now);
    osc.stop(now + 0.26);
  } else {
    osc.type = "sine";
    osc.frequency.setValueAtTime(523.25, now);
    osc.frequency.setValueAtTime(659.25, now + 0.1);
    osc.frequency.setValueAtTime(783.99, now + 0.2);
    gain.gain.setValueAtTime(gainLevel, now);
    gain.gain.exponentialRampToValueAtTime(0.001, now + 0.45);
    osc.start(now);
    osc.stop(now + 0.46);
  }
}
