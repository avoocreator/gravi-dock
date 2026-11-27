import React, { useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { NobiMood } from "../types/island";
import { useIslandStore } from "../store/useIslandStore";
import { isTauriRuntime } from "../lib/tauriBridge";
import { playNobiSound } from "../lib/soundEffects";

interface NobiCanvasProps {
  size?: number;
  mood: NobiMood;
  onMoodChange?: (mood: NobiMood) => void;
  onClick?: () => void;
}

const CYCLE_MOODS: NobiMood[] = [
  "happy",
  "love",
  "wink",
  "excited",
  "celebrate",
  "cool",
  "shy",
  "surprised",
  "charging",
  "wifi",
  "sleepy",
  "idle",
];

export const NobiCanvas: React.FC<NobiCanvasProps> = ({
  size = 44,
  mood,
  onMoodChange,
  onClick,
}) => {
  const settings = useIslandStore((s) => s.settings);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const pointerRef = useRef({ x: 0, y: 0, dist: 999 });
  const clickTimesRef = useRef<number[]>([]);
  const squishRef = useRef(0);
  const hoverRef = useRef(false);
  const moodRef = useRef<NobiMood>(mood);
  const settingsRef = useRef(settings);
  const petScoreRef = useRef(0);
  const lastPetXRef = useRef<number | null>(null);
  const lastPetDirRef = useRef(0);
  const cycleIdxRef = useRef(0);
  const tempMoodTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    moodRef.current = mood;
  }, [mood]);

  useEffect(() => {
    settingsRef.current = settings;
  }, [settings]);

  useEffect(() => {
    return () => {
      if (tempMoodTimerRef.current) clearTimeout(tempMoodTimerRef.current);
    };
  }, []);

  // Eye tracking
  useEffect(() => {
    const updatePointerFromClient = (clientX: number, clientY: number) => {
      if (!settingsRef.current.eye_tracking) {
        pointerRef.current = { x: 0, y: 0, dist: 999 };
        return;
      }
      const canvas = canvasRef.current;
      if (!canvas) return;
      const rect = canvas.getBoundingClientRect();
      if (rect.width <= 0 || rect.height <= 0) return;

      const cx = rect.left + rect.width / 2;
      const cy = rect.top + rect.height / 2;
      const vx = clientX - cx;
      const vy = clientY - cy;
      const dist = Math.hypot(vx, vy);
      if (dist < 1) {
        pointerRef.current = { x: 0, y: 0, dist: 0 };
        return;
      }
      const mag = Math.tanh(dist / 220);
      const angle = Math.atan2(vy, vx);
      pointerRef.current = {
        x: Math.cos(angle) * mag,
        y: Math.sin(angle) * mag,
        dist,
      };
    };

    const handleMove = (e: MouseEvent) => {
      updatePointerFromClient(e.clientX, e.clientY);
    };
    window.addEventListener("mousemove", handleMove);

    let unlistenCursor: (() => void) | undefined;
    if (isTauriRuntime()) {
      void listen<{ x: number; y: number }>("nobi:cursor-pos", (e) => {
        updatePointerFromClient(e.payload.x, e.payload.y);
      }).then((u) => {
        unlistenCursor = u;
      });
    }

    return () => {
      window.removeEventListener("mousemove", handleMove);
      if (unlistenCursor) unlistenCursor();
    };
  }, []);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const dpr = window.devicePixelRatio || 1;
    canvas.width = size * dpr;
    canvas.height = size * dpr;

    let rafId = 0;
    let blinkTimer = 2.4;
    let blinkProgress = 0;
    let lastTime = performance.now();
    let smoothEyeX = 0;
    let smoothEyeY = 0;

    const drawRoundedRect = (
      x: number,
      y: number,
      w: number,
      h: number,
      r: number
    ) => {
      const radius = Math.min(r, w / 2, h / 2);
      ctx.beginPath();
      ctx.moveTo(x + radius, y);
      ctx.lineTo(x + w - radius, y);
      ctx.quadraticCurveTo(x + w, y, x + w, y + radius);
      ctx.lineTo(x + w, y + h - radius);
      ctx.quadraticCurveTo(x + w, y + h, x + w - radius, y + h);
      ctx.lineTo(x + radius, y + h);
      ctx.quadraticCurveTo(x, y + h, x, y + h - radius);
      ctx.lineTo(x, y + radius);
      ctx.quadraticCurveTo(x, y, x + radius, y);
      ctx.closePath();
    };

    const drawHeart = (x: number, y: number, r: number) => {
      ctx.beginPath();
      ctx.moveTo(x, y + r * 0.75);
      ctx.bezierCurveTo(x - r * 1.25, y - r * 0.1, x - r * 0.9, y - r * 1.15, x, y - r * 0.35);
      ctx.bezierCurveTo(x + r * 0.9, y - r * 1.15, x + r * 1.25, y - r * 0.1, x, y + r * 0.75);
      ctx.closePath();
      ctx.fill();
    };

    const drawStar = (x: number, y: number, outerR: number, innerR: number) => {
      ctx.beginPath();
      for (let i = 0; i < 8; i++) {
        const rad = i % 2 === 0 ? outerR : innerR;
        const a = (i * Math.PI) / 4 - Math.PI / 2;
        const px = x + Math.cos(a) * rad;
        const py = y + Math.sin(a) * rad;
        if (i === 0) ctx.moveTo(px, py);
        else ctx.lineTo(px, py);
      }
      ctx.closePath();
      ctx.fill();
    };

    const drawBolt = (x: number, y: number, scale: number) => {
      ctx.beginPath();
      ctx.moveTo(x + 2 * scale, y - 9 * scale);
      ctx.lineTo(x - 6 * scale, y + 1 * scale);
      ctx.lineTo(x - 0.5 * scale, y + 1 * scale);
      ctx.lineTo(x - 2.5 * scale, y + 9 * scale);
      ctx.lineTo(x + 6 * scale, y - 1 * scale);
      ctx.lineTo(x + 0.5 * scale, y - 1 * scale);
      ctx.closePath();
      ctx.fill();
    };

    const render = (now: number) => {
      const dt = Math.min(0.05, (now - lastTime) / 1000);
      lastTime = now;
      const t = now / 1000;
      const activeMood = moodRef.current;
      const cfg = settingsRef.current;

      blinkTimer -= dt;
      if (blinkTimer <= 0) {
        blinkProgress += dt * 11.5;
        if (blinkProgress >= Math.PI) {
          blinkProgress = 0;
          blinkTimer = 2.0 + Math.random() * 3.2;
        }
      }
      const blinkScaleY = 1 - Math.sin(blinkProgress) * 0.92;

      squishRef.current *= Math.pow(0.035, dt);
      petScoreRef.current = Math.max(0, petScoreRef.current - dt * 1.4);

      const targetX = cfg.eye_tracking ? pointerRef.current.x : 0;
      const targetY = cfg.eye_tracking ? pointerRef.current.y : 0;
      smoothEyeX += (targetX - smoothEyeX) * Math.min(1, dt * 14);
      smoothEyeY += (targetY - smoothEyeY) * Math.min(1, dt * 14);

      ctx.save();
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, size, size);

      const cx = size / 2;
      const cy = size * 0.56;
      const s = size / 100;

      // Body motion
      let bobY = Math.sin(t * 2.6) * 1.5 * s;
      let rot = cfg.eye_tracking ? smoothEyeX * 0.055 : 0;
      let scaleX = 1 + squishRef.current * 0.2;
      let scaleY = 1 - squishRef.current * 0.17;

      if (activeMood === "dancing") {
        bobY = -Math.abs(Math.sin(t * 7.2)) * 4.5 * s;
        rot = Math.sin(t * 3.6) * 0.09;
      } else if (activeMood === "celebrate") {
        bobY = -Math.abs(Math.sin(t * 9.0)) * 5.2 * s;
        rot = Math.sin(t * 5.5) * 0.09;
      } else if (activeMood === "panic") {
        bobY = -Math.abs(Math.sin(t * 14)) * 3.8 * s;
        rot = Math.sin(t * 22) * 0.1;
      } else if (activeMood === "alert" || activeMood === "notif") {
        bobY = -Math.abs(Math.sin(t * 8.0)) * 3.0 * s;
      } else if (activeMood === "happy" || activeMood === "love" || activeMood === "battery_full") {
        bobY = -Math.abs(Math.sin(t * 5.8)) * 3.2 * s;
      } else if (activeMood === "excited" || activeMood === "charging") {
        bobY = -Math.abs(Math.sin(t * 8.5)) * 3.6 * s;
      } else if (activeMood === "dizzy") {
        rot = Math.sin(t * 10) * 0.13;
      } else if (activeMood === "sleepy" || activeMood === "low_battery") {
        bobY = Math.sin(t * 1.3) * 1.8 * s + 2.0 * s;
        rot = activeMood === "low_battery" ? -0.07 + Math.sin(t * 2.4) * 0.03 : Math.sin(t * 0.9) * 0.06;
      } else if (activeMood === "surprised" || activeMood === "capslock") {
        bobY = -2.8 * s;
        scaleX *= 0.96;
        scaleY *= 1.06;
      } else if (activeMood === "shy" || activeMood === "wifi_off") {
        rot = -0.08 + Math.sin(t * 2.5) * 0.03;
      }

      ctx.translate(cx, cy + bobY);
      ctx.rotate(rot);
      ctx.scale(scaleX, scaleY);

      // Drop mode
      if (activeMood === "drop") {
        ctx.save();
        const bx = -30 * s;
        const by = -26 * s;
        drawRoundedRect(bx, by, 60 * s, 52 * s, 14 * s);
        const bgGrad = ctx.createLinearGradient(0, by, 0, by + 52 * s);
        bgGrad.addColorStop(0, "#FFFFFF");
        bgGrad.addColorStop(1, "#DDD8F5");
        ctx.fillStyle = bgGrad;
        ctx.fill();
        ctx.strokeStyle = "#60A5FA";
        ctx.lineWidth = 2.5 * s;
        ctx.setLineDash([5 * s, 3 * s]);
        ctx.stroke();
        ctx.setLineDash([]);
        ctx.strokeStyle = "#1E1E2B";
        ctx.lineWidth = 3.5 * s;
        ctx.lineCap = "round";
        ctx.beginPath();
        ctx.moveTo(0, -10 * s);
        ctx.lineTo(0, 10 * s);
        ctx.moveTo(-9 * s, 2 * s);
        ctx.lineTo(0, 11 * s);
        ctx.lineTo(9 * s, 2 * s);
        ctx.stroke();
        ctx.restore();
        ctx.restore();
        rafId = requestAnimationFrame(render);
        return;
      }

      const earWiggle = !cfg.ear_animations
        ? 0
        : activeMood === "dancing" || activeMood === "celebrate"
        ? Math.sin(t * 8.2) * 0.17
        : activeMood === "excited" || activeMood === "charging"
        ? Math.sin(t * 12) * 0.14
        : activeMood === "sleepy" || activeMood === "shy" || activeMood === "low_battery" || activeMood === "wifi_off"
        ? -0.24
        : activeMood === "surprised" || activeMood === "capslock" || activeMood === "notif"
        ? 0.2
        : hoverRef.current
        ? Math.sin(t * 9) * 0.09
        : activeMood === "squish"
        ? -0.18
        : Math.sin(t * 1.9) * 0.028;

      const alertPulse =
        activeMood === "alert" || activeMood === "low_battery" || activeMood === "panic"
          ? 0.55 + 0.45 * Math.sin(t * 10)
          : 1;

      // Left ear
      ctx.save();
      ctx.translate(-22 * s, -16 * s);
      ctx.rotate(-0.18 - earWiggle);
      ctx.beginPath();
      ctx.moveTo(14 * s, -6 * s);
      ctx.quadraticCurveTo(-8 * s, -28 * s, -22 * s, -22 * s);
      ctx.quadraticCurveTo(-30 * s, -4 * s, -18 * s, 18 * s);
      ctx.closePath();
      ctx.fillStyle = "#262635";
      ctx.fill();

      ctx.save();
      ctx.globalAlpha = alertPulse;
      ctx.beginPath();
      ctx.moveTo(-18 * s, -21 * s);
      ctx.quadraticCurveTo(-31 * s, -4 * s, -20 * s, 17 * s);
      ctx.quadraticCurveTo(-13 * s, 14 * s, -10 * s, 5 * s);
      ctx.quadraticCurveTo(-16 * s, -8 * s, -18 * s, -21 * s);
      ctx.closePath();
      const leftEarGrad = ctx.createLinearGradient(-26 * s, -22 * s, -16 * s, 18 * s);
      if (activeMood === "low_battery" || activeMood === "panic") {
        leftEarGrad.addColorStop(0, "#EF4444");
        leftEarGrad.addColorStop(1, "#F87171");
      } else if (activeMood === "charging" || activeMood === "battery_full") {
        leftEarGrad.addColorStop(0, "#10B981");
        leftEarGrad.addColorStop(1, "#4ADE80");
      } else {
        leftEarGrad.addColorStop(0, "#2563EB");
        leftEarGrad.addColorStop(0.55, "#38BDF8");
        leftEarGrad.addColorStop(1, "#4ADE80");
      }
      ctx.fillStyle = leftEarGrad;
      ctx.fill();
      ctx.restore();
      ctx.restore();

      // Right ear
      ctx.save();
      ctx.translate(22 * s, -16 * s);
      ctx.rotate(0.18 + earWiggle);
      ctx.beginPath();
      ctx.moveTo(-14 * s, -6 * s);
      ctx.quadraticCurveTo(8 * s, -28 * s, 22 * s, -22 * s);
      ctx.quadraticCurveTo(30 * s, -4 * s, 18 * s, 18 * s);
      ctx.closePath();
      ctx.fillStyle = "#262635";
      ctx.fill();

      ctx.save();
      ctx.globalAlpha = alertPulse;
      ctx.beginPath();
      ctx.moveTo(18 * s, -21 * s);
      ctx.quadraticCurveTo(31 * s, -4 * s, 20 * s, 17 * s);
      ctx.quadraticCurveTo(13 * s, 14 * s, 10 * s, 5 * s);
      ctx.quadraticCurveTo(16 * s, -8 * s, 18 * s, -21 * s);
      ctx.closePath();
      const rightEarGrad = ctx.createLinearGradient(26 * s, -22 * s, 16 * s, 18 * s);
      if (activeMood === "low_battery" || activeMood === "panic") {
        rightEarGrad.addColorStop(0, "#DC2626");
        rightEarGrad.addColorStop(1, "#FB7185");
      } else if (activeMood === "charging" || activeMood === "battery_full") {
        rightEarGrad.addColorStop(0, "#34D399");
        rightEarGrad.addColorStop(1, "#A3E635");
      } else {
        rightEarGrad.addColorStop(0, "#FF4D2E");
        rightEarGrad.addColorStop(0.55, "#FB923C");
        rightEarGrad.addColorStop(1, "#FACC15");
      }
      ctx.fillStyle = rightEarGrad;
      ctx.fill();
      ctx.restore();
      ctx.restore();

      // Helmet body
      ctx.beginPath();
      ctx.moveTo(0, -32 * s);
      ctx.bezierCurveTo(28 * s, -32 * s, 42 * s, -10 * s, 40 * s, 12 * s);
      ctx.bezierCurveTo(38 * s, 28 * s, 22 * s, 32 * s, 0, 32 * s);
      ctx.bezierCurveTo(-22 * s, 32 * s, -38 * s, 28 * s, -40 * s, 12 * s);
      ctx.bezierCurveTo(-42 * s, -10 * s, -28 * s, -32 * s, 0, -32 * s);
      ctx.closePath();

      const bodyGrad = ctx.createLinearGradient(0, -32 * s, 0, 32 * s);
      bodyGrad.addColorStop(0, "#FFFFFF");
      bodyGrad.addColorStop(0.68, "#F5F3FF");
      bodyGrad.addColorStop(1, "#DBD5F3");
      ctx.fillStyle = bodyGrad;
      ctx.fill();

      // Dark visor
      ctx.beginPath();
      ctx.moveTo(0, -21 * s);
      ctx.bezierCurveTo(21 * s, -21 * s, 31 * s, -5 * s, 30 * s, 10 * s);
      ctx.bezierCurveTo(29 * s, 22 * s, 18 * s, 24 * s, 0, 24 * s);
      ctx.bezierCurveTo(-18 * s, 24 * s, -29 * s, 22 * s, -30 * s, 10 * s);
      ctx.bezierCurveTo(-31 * s, -5 * s, -21 * s, -21 * s, 0, -21 * s);
      ctx.closePath();

      const visorGrad = ctx.createLinearGradient(0, -21 * s, 0, 24 * s);
      visorGrad.addColorStop(0, "#242436");
      visorGrad.addColorStop(1, "#13131F");
      ctx.fillStyle = visorGrad;
      ctx.fill();

      ctx.save();
      ctx.strokeStyle = "rgba(255, 255, 255, 0.22)";
      ctx.lineWidth = 3.2 * s;
      ctx.lineCap = "round";
      ctx.beginPath();
      ctx.moveTo(-24 * s, 0);
      ctx.quadraticCurveTo(-22 * s, -12 * s, -11 * s, -15 * s);
      ctx.stroke();
      ctx.restore();

      // LED eyes
      const eyeOffsetX = smoothEyeX * 6.0 * s;
      const eyeOffsetY = smoothEyeY * 4.2 * s;
      const leftEyeX = -11.5 * s + eyeOffsetX;
      const rightEyeX = 11.5 * s + eyeOffsetX;
      const eyeY = 2.5 * s + eyeOffsetY;

      if (
        activeMood === "happy" ||
        activeMood === "love" ||
        activeMood === "shy" ||
        activeMood === "wink" ||
        activeMood === "celebrate"
      ) {
        ctx.save();
        ctx.fillStyle =
          activeMood === "shy" || activeMood === "love"
            ? "rgba(251, 113, 133, 0.48)"
            : "rgba(251, 113, 133, 0.28)";
        ctx.beginPath();
        ctx.ellipse(-18 * s + eyeOffsetX * 0.4, 12 * s + eyeOffsetY * 0.4, 4.5 * s, 2.5 * s, 0, 0, Math.PI * 2);
        ctx.ellipse(18 * s + eyeOffsetX * 0.4, 12 * s + eyeOffsetY * 0.4, 4.5 * s, 2.5 * s, 0, 0, Math.PI * 2);
        ctx.fill();
        ctx.restore();
      }

      ctx.save();
      const glowColor =
        activeMood === "low_battery" || activeMood === "wifi_off" || activeMood === "panic"
          ? "rgba(248, 113, 113, 0.95)"
          : activeMood === "charging" || activeMood === "battery_full"
          ? "rgba(74, 222, 128, 0.95)"
          : activeMood === "wifi" || activeMood === "notif" || activeMood === "capslock"
          ? "rgba(56, 189, 248, 0.95)"
          : activeMood === "alert"
          ? "rgba(251, 146, 60, 0.92)"
          : activeMood === "love" || activeMood === "shy"
          ? "rgba(251, 113, 133, 0.9)"
          : activeMood === "excited" || activeMood === "celebrate"
          ? "rgba(250, 204, 21, 0.95)"
          : activeMood === "cool" || activeMood === "focus" || activeMood === "thinking"
          ? "rgba(56, 189, 248, 0.92)"
          : "rgba(220, 238, 255, 0.85)";

      const ledColor =
        activeMood === "low_battery"
          ? "#F87171"
          : activeMood === "wifi_off"
          ? "#FB923C"
          : activeMood === "charging" || activeMood === "battery_full"
          ? "#4ADE80"
          : activeMood === "wifi" || activeMood === "notif" || activeMood === "capslock" || activeMood === "cool"
          ? "#38BDF8"
          : activeMood === "alert"
          ? "#FEF3C7"
          : activeMood === "love"
          ? "#FB7185"
          : activeMood === "excited" || activeMood === "celebrate"
          ? "#FEF08A"
          : "#FFFFFF";

      ctx.shadowColor = glowColor;
      ctx.shadowBlur = 7.5 * s;
      ctx.fillStyle = ledColor;
      ctx.strokeStyle = ledColor;

      if (activeMood === "panic") {
        ctx.lineWidth = 3.0 * s;
        for (const ex of [leftEyeX, rightEyeX]) {
          ctx.beginPath();
          ctx.arc(ex, eyeY, 6.2 * s, 0, Math.PI * 2);
          ctx.stroke();
          ctx.beginPath();
          ctx.arc(ex + Math.sin(t * 20) * 1.4 * s, eyeY, 2.0 * s, 0, Math.PI * 2);
          ctx.fill();
        }
        const dropY = -12 * s + ((t * 16) % 10) * s;
        ctx.fillStyle = "#38BDF8";
        ctx.shadowColor = "rgba(56, 189, 248, 0.9)";
        ctx.beginPath();
        ctx.moveTo(21 * s, dropY - 4 * s);
        ctx.quadraticCurveTo(24.5 * s, dropY + 1 * s, 21 * s, dropY + 3.5 * s);
        ctx.quadraticCurveTo(17.5 * s, dropY + 1 * s, 21 * s, dropY - 4 * s);
        ctx.fill();
      } else if (activeMood === "charging") {
        const pulse = 1 + Math.sin(t * 9) * 0.12;
        drawBolt(0 + eyeOffsetX * 0.35, 2 * s + eyeOffsetY * 0.3, s * 1.45 * pulse);
      } else if (activeMood === "low_battery") {
        const blinkOn = Math.sin(t * 7.5) > -0.35;
        ctx.globalAlpha = blinkOn ? 1 : 0.35;
        ctx.lineWidth = 2.6 * s;
        drawRoundedRect(-17 * s, -6 * s, 31 * s, 16 * s, 4 * s);
        ctx.stroke();
        drawRoundedRect(14.5 * s, -2 * s, 3.5 * s, 8 * s, 1.5 * s);
        ctx.fill();
        drawRoundedRect(-13.5 * s, -2.5 * s, 6.5 * s, 9 * s, 2 * s);
        ctx.fill();
      } else if (activeMood === "battery_full") {
        ctx.lineWidth = 2.4 * s;
        drawRoundedRect(-17 * s, -6 * s, 31 * s, 16 * s, 4 * s);
        ctx.stroke();
        drawRoundedRect(14.5 * s, -2 * s, 3.5 * s, 8 * s, 1.5 * s);
        ctx.fill();
        drawRoundedRect(-13.5 * s, -2.5 * s, 24 * s, 9 * s, 2 * s);
        ctx.fill();
      } else if (activeMood === "wifi") {
        ctx.lineWidth = 2.8 * s;
        ctx.lineCap = "round";
        const wy = 11 * s;
        const phase = (t * 2.2) % 3;
        ctx.beginPath();
        ctx.arc(0, wy, 2.8 * s, 0, Math.PI * 2);
        ctx.fill();
        for (let i = 1; i <= 3; i++) {
          ctx.globalAlpha = phase >= i - 1 ? 1 : 0.28;
          ctx.beginPath();
          ctx.arc(0, wy, (i * 6.5 + 1) * s, Math.PI * 1.22, Math.PI * 1.78);
          ctx.stroke();
        }
      } else if (activeMood === "wifi_off") {
        ctx.lineWidth = 3.1 * s;
        ctx.lineCap = "round";
        for (const ex of [leftEyeX, rightEyeX]) {
          ctx.beginPath();
          ctx.moveTo(ex - 4.2 * s, eyeY - 4.2 * s);
          ctx.lineTo(ex + 4.2 * s, eyeY + 4.2 * s);
          ctx.moveTo(ex + 4.2 * s, eyeY - 4.2 * s);
          ctx.lineTo(ex - 4.2 * s, eyeY + 4.2 * s);
          ctx.stroke();
        }
      } else if (activeMood === "capslock") {
        ctx.lineWidth = 3.0 * s;
        ctx.lineCap = "round";
        ctx.lineJoin = "round";
        ctx.beginPath();
        ctx.moveTo(-12 * s, 9 * s);
        ctx.lineTo(-12 * s, -6 * s);
        ctx.moveTo(-17 * s, -1 * s);
        ctx.lineTo(-12 * s, -6 * s);
        ctx.lineTo(-7 * s, -1 * s);
        ctx.stroke();
        ctx.font = `bold ${Math.round(21 * s)}px system-ui, sans-serif`;
        ctx.textAlign = "center";
        ctx.textBaseline = "middle";
        ctx.fillText("A", 9 * s, 3 * s);
      } else if (activeMood === "celebrate") {
        const pulse = 1 + Math.sin(t * 11) * 0.16;
        drawStar(leftEyeX, eyeY, 7.2 * s * pulse, 2.8 * s * pulse);
        drawStar(rightEyeX, eyeY, 7.2 * s * pulse, 2.8 * s * pulse);
        const colors = ["#38BDF8", "#4ADE80", "#FB7185", "#FACC15"];
        for (let i = 0; i < 4; i++) {
          const ang = t * 3.5 + (i * Math.PI) / 2;
          const px = Math.cos(ang) * 20 * s;
          const py = Math.sin(ang) * 11 * s - 2 * s;
          ctx.fillStyle = colors[i];
          ctx.beginPath();
          ctx.arc(px, py, 2.1 * s, 0, Math.PI * 2);
          ctx.fill();
        }
      } else if (activeMood === "notif") {
        const eyeW = 7.4 * s;
        const eyeH = 14.2 * s * blinkScaleY;
        drawRoundedRect(leftEyeX - eyeW / 2, eyeY - eyeH / 2, eyeW, Math.max(2 * s, eyeH), eyeW / 2);
        ctx.fill();
        drawRoundedRect(rightEyeX - eyeW / 2, eyeY - eyeH / 2, eyeW, Math.max(2 * s, eyeH), eyeW / 2);
        ctx.fill();
        ctx.fillStyle = "#FB7185";
        ctx.shadowColor = "rgba(251, 113, 133, 0.9)";
        const nr = (3.4 + Math.sin(t * 8) * 0.8) * s;
        ctx.beginPath();
        ctx.arc(20 * s, -12 * s, nr, 0, Math.PI * 2);
        ctx.fill();
      } else if (activeMood === "happy") {
        ctx.lineWidth = 3.6 * s;
        ctx.lineCap = "round";
        for (const ex of [leftEyeX, rightEyeX]) {
          ctx.beginPath();
          ctx.arc(ex, eyeY + 2 * s, 5.5 * s, Math.PI * 1.15, Math.PI * 1.85);
          ctx.stroke();
        }
      } else if (activeMood === "love") {
        const pulse = 1 + Math.sin(t * 7) * 0.12;
        drawHeart(leftEyeX, eyeY, 6.2 * s * pulse);
        drawHeart(rightEyeX, eyeY, 6.2 * s * pulse);
      } else if (activeMood === "excited") {
        const pulse = 1 + Math.sin(t * 11) * 0.15;
        drawStar(leftEyeX, eyeY, 7.2 * s * pulse, 2.8 * s * pulse);
        drawStar(rightEyeX, eyeY, 7.2 * s * pulse, 2.8 * s * pulse);
      } else if (activeMood === "wink") {
        const eyeW = 7.2 * s;
        const eyeH = 14.5 * s;
        drawRoundedRect(leftEyeX - eyeW / 2, eyeY - eyeH / 2, eyeW, eyeH, eyeW / 2);
        ctx.fill();
        ctx.lineWidth = 3.3 * s;
        ctx.lineCap = "round";
        ctx.lineJoin = "round";
        ctx.beginPath();
        ctx.moveTo(rightEyeX + 4.5 * s, eyeY - 4 * s);
        ctx.lineTo(rightEyeX - 3.5 * s, eyeY);
        ctx.lineTo(rightEyeX + 4.5 * s, eyeY + 4 * s);
        ctx.stroke();
      } else if (activeMood === "sleepy") {
        ctx.lineWidth = 3.2 * s;
        ctx.lineCap = "round";
        for (const ex of [leftEyeX, rightEyeX]) {
          ctx.beginPath();
          ctx.moveTo(ex - 5 * s, eyeY + 1.5 * s);
          ctx.quadraticCurveTo(ex, eyeY + 4.5 * s, ex + 5 * s, eyeY + 1.5 * s);
          ctx.stroke();
        }
        const zOff = (t * 14) % 14;
        ctx.globalAlpha = Math.max(0, 1 - zOff / 14);
        ctx.font = `bold ${Math.max(8, 11 * s)}px sans-serif`;
        ctx.fillText("z", 14 * s, (-6 - zOff * 0.7) * s);
      } else if (activeMood === "thinking") {
        for (let i = -1; i <= 1; i++) {
          const phase = Math.sin(t * 6.5 - i * 0.9);
          const rDot = (3.4 + Math.max(0, phase) * 1.6) * s;
          ctx.beginPath();
          ctx.arc(i * 10.5 * s + eyeOffsetX * 0.5, eyeY - Math.max(0, phase) * 2.5 * s, rDot, 0, Math.PI * 2);
          ctx.fill();
        }
      } else if (activeMood === "surprised") {
        ctx.lineWidth = 3.2 * s;
        for (const ex of [leftEyeX, rightEyeX]) {
          ctx.beginPath();
          ctx.arc(ex, eyeY, 5.6 * s, 0, Math.PI * 2);
          ctx.stroke();
          ctx.beginPath();
          ctx.arc(ex, eyeY, 2.2 * s, 0, Math.PI * 2);
          ctx.fill();
        }
      } else if (activeMood === "cool") {
        drawRoundedRect(-21 * s + eyeOffsetX * 0.4, eyeY - 5 * s, 42 * s, 10 * s, 5 * s);
        ctx.fillStyle = "#0F172A";
        ctx.fill();
        ctx.lineWidth = 2.2 * s;
        ctx.strokeStyle = "#38BDF8";
        ctx.stroke();
        const scanX = -15 * s + ((t * 36) % 30) * s;
        ctx.fillStyle = "rgba(56, 189, 248, 0.85)";
        drawRoundedRect(scanX + eyeOffsetX * 0.4, eyeY - 3 * s, 8 * s, 6 * s, 3 * s);
        ctx.fill();
      } else if (activeMood === "shy") {
        const eyeW = 6.2 * s;
        const eyeH = 11.5 * s * blinkScaleY;
        ctx.save();
        ctx.translate(leftEyeX, eyeY);
        ctx.rotate(0.18);
        drawRoundedRect(-eyeW / 2, -eyeH / 2, eyeW, Math.max(2 * s, eyeH), eyeW / 2);
        ctx.fill();
        ctx.restore();
        ctx.save();
        ctx.translate(rightEyeX, eyeY);
        ctx.rotate(-0.18);
        drawRoundedRect(-eyeW / 2, -eyeH / 2, eyeW, Math.max(2 * s, eyeH), eyeW / 2);
        ctx.fill();
        ctx.restore();
      } else if (activeMood === "squish") {
        ctx.lineWidth = 3.2 * s;
        ctx.lineCap = "round";
        ctx.lineJoin = "round";
        ctx.beginPath();
        ctx.moveTo(leftEyeX - 4 * s, eyeY - 4 * s);
        ctx.lineTo(leftEyeX + 3 * s, eyeY);
        ctx.lineTo(leftEyeX - 4 * s, eyeY + 4 * s);
        ctx.moveTo(rightEyeX + 4 * s, eyeY - 4 * s);
        ctx.lineTo(rightEyeX - 3 * s, eyeY);
        ctx.lineTo(rightEyeX + 4 * s, eyeY + 4 * s);
        ctx.stroke();
      } else if (activeMood === "dizzy") {
        ctx.lineWidth = 2.8 * s;
        ctx.lineCap = "round";
        for (const ex of [leftEyeX, rightEyeX]) {
          ctx.save();
          ctx.translate(ex, eyeY);
          ctx.rotate(t * 8);
          ctx.beginPath();
          ctx.arc(0, 0, 4.6 * s, 0, Math.PI * 1.45);
          ctx.stroke();
          ctx.restore();
        }
      } else if (activeMood === "dancing") {
        ctx.lineWidth = 3.4 * s;
        ctx.lineCap = "round";
        for (const ex of [leftEyeX, rightEyeX]) {
          ctx.beginPath();
          ctx.arc(ex, eyeY + 1.5 * s, 5.2 * s, Math.PI * 1.15, Math.PI * 1.85);
          ctx.stroke();
        }
        const nOff = (t * 18) % 14;
        ctx.globalAlpha = Math.max(0, 1 - nOff / 14);
        ctx.fillStyle = "#38BDF8";
        ctx.font = `bold ${Math.max(8, 12 * s)}px sans-serif`;
        ctx.fillText("♪", 15 * s, (-5 - nOff * 0.6) * s);
      } else {
        const eyeW = (hoverRef.current ? 7.8 : 7.0) * s;
        const eyeH =
          (activeMood === "focus" ? 10.5 : hoverRef.current ? 16.5 : 14.8) *
          s *
          blinkScaleY;
        drawRoundedRect(
          leftEyeX - eyeW / 2,
          eyeY - eyeH / 2,
          eyeW,
          Math.max(2 * s, eyeH),
          eyeW / 2
        );
        ctx.fill();
        drawRoundedRect(
          rightEyeX - eyeW / 2,
          eyeY - eyeH / 2,
          eyeW,
          Math.max(2 * s, eyeH),
          eyeW / 2
        );
        ctx.fill();
      }

      ctx.restore();
      ctx.restore();
      rafId = requestAnimationFrame(render);
    };

    rafId = requestAnimationFrame(render);
    return () => cancelAnimationFrame(rafId);
  }, [size]);

  const setTempMood = (next: NobiMood, durationMs: number) => {
    if (tempMoodTimerRef.current) clearTimeout(tempMoodTimerRef.current);
    onMoodChange?.(next);
    tempMoodTimerRef.current = setTimeout(() => {
      if (moodRef.current === next) onMoodChange?.("idle");
    }, durationMs);
  };

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    const cfg = settingsRef.current;
    squishRef.current = 1;

    if (!cfg.squish_dizzy) {
      playNobiSound("pop", cfg.sound_enabled, cfg.sound_volume);
      onClick?.();
      return;
    }

    const now = Date.now();
    clickTimesRef.current = [...clickTimesRef.current.filter((t) => now - t < 800), now];

    if (clickTimesRef.current.length >= 3) {
      clickTimesRef.current = [];
      playNobiSound("dizzy", cfg.sound_enabled, cfg.sound_volume);
      setTempMood("dizzy", 2500);
    } else if (clickTimesRef.current.length === 2) {
      playNobiSound("happy", cfg.sound_enabled, cfg.sound_volume);
      setTempMood("love", 1800);
    } else {
      playNobiSound("pop", cfg.sound_enabled, cfg.sound_volume);
      const reactive: NobiMood[] = ["squish", "wink", "happy", "surprised"];
      const pick = reactive[Math.floor(Math.random() * reactive.length)];
      setTempMood(pick, 850);
    }
    onClick?.();
  };

  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const cfg = settingsRef.current;
    playNobiSound("pop", cfg.sound_enabled, cfg.sound_volume);
    const next = CYCLE_MOODS[cycleIdxRef.current % CYCLE_MOODS.length];
    cycleIdxRef.current += 1;
    squishRef.current = 0.7;
    if (tempMoodTimerRef.current) clearTimeout(tempMoodTimerRef.current);
    onMoodChange?.(next);
  };

  const handleCanvasMouseMove = (e: React.MouseEvent) => {
    const x = e.clientX;
    if (lastPetXRef.current !== null) {
      const dx = x - lastPetXRef.current;
      if (Math.abs(dx) >= 4) {
        const dir = dx > 0 ? 1 : -1;
        if (lastPetDirRef.current !== 0 && dir !== lastPetDirRef.current) {
          petScoreRef.current += 1;
          if (petScoreRef.current >= 3 && moodRef.current === "idle") {
            petScoreRef.current = 0;
            const cfg = settingsRef.current;
            playNobiSound("happy", cfg.sound_enabled, cfg.sound_volume);
            setTempMood(Math.random() > 0.45 ? "love" : "shy", 1800);
          }
        }
        lastPetDirRef.current = dir;
        lastPetXRef.current = x;
      }
    } else {
      lastPetXRef.current = x;
    }
  };

  return (
    <canvas
      ref={canvasRef}
      style={{ width: size, height: size }}
      className="cursor-pointer shrink-0 select-none"
      onMouseEnter={() => {
        hoverRef.current = true;
        lastPetXRef.current = null;
        lastPetDirRef.current = 0;
      }}
      onMouseLeave={() => {
        hoverRef.current = false;
        lastPetXRef.current = null;
        lastPetDirRef.current = 0;
      }}
      onMouseMove={handleCanvasMouseMove}
      onClick={handleClick}
      onContextMenu={handleContextMenu}
      title="Klik: reaksi · 2x: love · 3x: pusing · Elus: malu · Klik kanan: ganti ekspresi"
    />
  );
};
