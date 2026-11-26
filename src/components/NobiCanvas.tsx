import React, { useEffect, useRef } from "react";
import { NobiMood } from "../types/island";

interface NobiCanvasProps {
  size?: number;
  mood: NobiMood;
  onMoodChange?: (mood: NobiMood) => void;
  onClick?: () => void;
}

/**
 * Procedural 60 fps Canvas 2D Robot-Cat Companion
 * Faithfully modeled on the user's custom robot-cat design:
 * - Soft white-lavender rounded helmet body
 * - Glossy dark visor with subtle curved specular reflection
 * - Dual dark cat ears with left blue->mint gradient and right coral->amber gradient
 * - Glowing white LED pill eyes with cursor eye-tracking, blinking, squish, dizzy, dancing & box-drop states
 */
export const NobiCanvas: React.FC<NobiCanvasProps> = ({
  size = 44,
  mood,
  onMoodChange,
  onClick,
}) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const pointerRef = useRef({ x: 0, y: 0 });
  const clickTimesRef = useRef<number[]>([]);
  const squishRef = useRef(0);
  const hoverRef = useRef(false);
  const moodRef = useRef<NobiMood>(mood);

  useEffect(() => {
    moodRef.current = mood;
  }, [mood]);

  useEffect(() => {
    const handleMove = (e: MouseEvent) => {
      const canvas = canvasRef.current;
      if (!canvas) return;
      const rect = canvas.getBoundingClientRect();
      const cx = rect.left + rect.width / 2;
      const cy = rect.top + rect.height / 2;
      const dx = Math.max(-1, Math.min(1, (e.clientX - cx) / 140));
      const dy = Math.max(-1, Math.min(1, (e.clientY - cy) / 140));
      pointerRef.current = { x: dx, y: dy };
    };
    window.addEventListener("mousemove", handleMove);
    return () => window.removeEventListener("mousemove", handleMove);
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

    const render = (now: number) => {
      const dt = Math.min(0.05, (now - lastTime) / 1000);
      lastTime = now;
      const t = now / 1000;
      const activeMood = moodRef.current;

      // Blink state machine
      blinkTimer -= dt;
      if (blinkTimer <= 0) {
        blinkProgress += dt * 11;
        if (blinkProgress >= Math.PI) {
          blinkProgress = 0;
          blinkTimer = 2.2 + Math.random() * 3.0;
        }
      }
      const blinkScaleY = 1 - Math.sin(blinkProgress) * 0.92;

      // Spring decay for squish
      squishRef.current *= Math.pow(0.04, dt);

      // Smooth eye tracking
      smoothEyeX += (pointerRef.current.x - smoothEyeX) * Math.min(1, dt * 12);
      smoothEyeY += (pointerRef.current.y - smoothEyeY) * Math.min(1, dt * 12);

      ctx.save();
      ctx.scale(dpr, dpr);
      ctx.clearRect(0, 0, size, size);

      const cx = size / 2;
      const cy = size * 0.56;
      const s = size / 100; // Normalized 100x100 coordinate space

      // Idle breathing / dancing / alert bounce
      let bobY = Math.sin(t * 2.6) * 1.6 * s;
      let rot = 0;
      let scaleX = 1 + squishRef.current * 0.18;
      let scaleY = 1 - squishRef.current * 0.16;

      if (activeMood === "dancing") {
        bobY = -Math.abs(Math.sin(t * 7.2)) * 4.5 * s;
        rot = Math.sin(t * 3.6) * 0.08;
      } else if (activeMood === "alert") {
        bobY = -Math.abs(Math.sin(t * 8.5)) * 3.2 * s;
      } else if (activeMood === "happy") {
        bobY = -Math.abs(Math.sin(t * 6.0)) * 3.5 * s;
      } else if (activeMood === "dizzy") {
        rot = Math.sin(t * 10) * 0.11;
      }

      ctx.translate(cx, cy + bobY);
      ctx.rotate(rot);
      ctx.scale(scaleX, scaleY);

      // Drop-zone box mode when dragging a file onto the island
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

        // Down arrow icon
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

      // Ear wiggle dynamics
      const earWiggle =
        activeMood === "dancing"
          ? Math.sin(t * 7.2) * 0.14
          : hoverRef.current
          ? Math.sin(t * 9) * 0.08
          : activeMood === "squish"
          ? -0.18
          : Math.sin(t * 1.9) * 0.025;

      const alertPulse =
        activeMood === "alert" ? 0.65 + 0.35 * Math.sin(t * 10) : 1;

      // === LEFT CAT EAR (Dark interior + Blue->Mint outer gradient) ===
      ctx.save();
      ctx.translate(-22 * s, -16 * s);
      ctx.rotate(-0.18 - earWiggle);

      // Dark inner ear silhouette
      ctx.beginPath();
      ctx.moveTo(14 * s, -6 * s);
      ctx.quadraticCurveTo(-8 * s, -28 * s, -22 * s, -22 * s);
      ctx.quadraticCurveTo(-30 * s, -4 * s, -18 * s, 18 * s);
      ctx.closePath();
      ctx.fillStyle = "#262635";
      ctx.fill();

      // Left outer gradient accent (Blue -> Cyan -> Mint Green)
      ctx.save();
      ctx.globalAlpha = alertPulse;
      ctx.beginPath();
      ctx.moveTo(-18 * s, -21 * s);
      ctx.quadraticCurveTo(-31 * s, -4 * s, -20 * s, 17 * s);
      ctx.quadraticCurveTo(-13 * s, 14 * s, -10 * s, 5 * s);
      ctx.quadraticCurveTo(-16 * s, -8 * s, -18 * s, -21 * s);
      ctx.closePath();
      const leftEarGrad = ctx.createLinearGradient(-26 * s, -22 * s, -16 * s, 18 * s);
      leftEarGrad.addColorStop(0, "#2563EB");
      leftEarGrad.addColorStop(0.55, "#38BDF8");
      leftEarGrad.addColorStop(1, "#4ADE80");
      ctx.fillStyle = leftEarGrad;
      ctx.fill();
      ctx.restore();
      ctx.restore();

      // === RIGHT CAT EAR (Dark interior + Coral/Red-Orange->Amber outer gradient) ===
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

      // Right outer gradient accent (Coral Red -> Orange -> Golden Yellow)
      ctx.save();
      ctx.globalAlpha = alertPulse;
      ctx.beginPath();
      ctx.moveTo(18 * s, -21 * s);
      ctx.quadraticCurveTo(31 * s, -4 * s, 20 * s, 17 * s);
      ctx.quadraticCurveTo(13 * s, 14 * s, 10 * s, 5 * s);
      ctx.quadraticCurveTo(16 * s, -8 * s, 18 * s, -21 * s);
      ctx.closePath();
      const rightEarGrad = ctx.createLinearGradient(26 * s, -22 * s, 16 * s, 18 * s);
      rightEarGrad.addColorStop(0, "#FF4D2E");
      rightEarGrad.addColorStop(0.55, "#FB923C");
      rightEarGrad.addColorStop(1, "#FACC15");
      ctx.fillStyle = rightEarGrad;
      ctx.fill();
      ctx.restore();
      ctx.restore();

      // === MAIN HELMET BODY (White to Soft Lavender) ===
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

      // === GLOSSY DARK VISOR ===
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

      // Visor top-left curved specular highlight
      ctx.save();
      ctx.strokeStyle = "rgba(255, 255, 255, 0.22)";
      ctx.lineWidth = 3.2 * s;
      ctx.lineCap = "round";
      ctx.beginPath();
      ctx.moveTo(-24 * s, 0);
      ctx.quadraticCurveTo(-22 * s, -12 * s, -11 * s, -15 * s);
      ctx.stroke();
      ctx.restore();

      // === GLOWING LED EYES INSIDE VISOR ===
      const eyeOffsetX = smoothEyeX * 5.2 * s;
      const eyeOffsetY = smoothEyeY * 3.8 * s;

      ctx.save();
      ctx.shadowColor =
        activeMood === "alert"
          ? "rgba(251, 146, 60, 0.9)"
          : activeMood === "focus"
          ? "rgba(96, 165, 250, 0.9)"
          : "rgba(220, 238, 255, 0.85)";
      ctx.shadowBlur = 7 * s;
      ctx.fillStyle = activeMood === "alert" ? "#FEF3C7" : "#FFFFFF";
      ctx.strokeStyle = activeMood === "alert" ? "#FEF3C7" : "#FFFFFF";

      const leftEyeX = -11.5 * s + eyeOffsetX;
      const rightEyeX = 11.5 * s + eyeOffsetX;
      const eyeY = 2.5 * s + eyeOffsetY;

      if (activeMood === "happy") {
        // Happy ^ ^ arcs
        ctx.lineWidth = 3.6 * s;
        ctx.lineCap = "round";
        for (const ex of [leftEyeX, rightEyeX]) {
          ctx.beginPath();
          ctx.arc(ex, eyeY + 2 * s, 5.5 * s, Math.PI * 1.15, Math.PI * 1.85);
          ctx.stroke();
        }
      } else if (activeMood === "squish") {
        // Squish > < eyes
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
        // Spinning X / ring eyes
        ctx.lineWidth = 2.8 * s;
        ctx.lineCap = "round";
        for (const ex of [leftEyeX, rightEyeX]) {
          ctx.save();
          ctx.translate(ex, eyeY);
          ctx.rotate(t * 7);
          ctx.beginPath();
          ctx.arc(0, 0, 4.5 * s, 0, Math.PI * 1.45);
          ctx.stroke();
          ctx.restore();
        }
      } else {
        // Signature glowing vertical pill LED eyes
        const eyeW = (hoverRef.current ? 7.8 : 7.0) * s;
        const eyeH =
          (activeMood === "focus" ? 11.5 : hoverRef.current ? 16.5 : 14.8) *
          s *
          blinkScaleY;

        drawRoundedRect(leftEyeX - eyeW / 2, eyeY - eyeH / 2, eyeW, Math.max(2 * s, eyeH), eyeW / 2);
        ctx.fill();

        drawRoundedRect(rightEyeX - eyeW / 2, eyeY - eyeH / 2, eyeW, Math.max(2 * s, eyeH), eyeW / 2);
        ctx.fill();
      }

      ctx.restore();
      ctx.restore();
      rafId = requestAnimationFrame(render);
    };

    rafId = requestAnimationFrame(render);
    return () => cancelAnimationFrame(rafId);
  }, [size]);

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    const now = Date.now();
    clickTimesRef.current = [...clickTimesRef.current.filter((t) => now - t < 750), now];
    squishRef.current = 1;

    if (clickTimesRef.current.length >= 3) {
      clickTimesRef.current = [];
      onMoodChange?.("dizzy");
      setTimeout(() => onMoodChange?.("idle"), 2400);
    } else {
      onMoodChange?.("squish");
      setTimeout(() => {
        if (moodRef.current === "squish") onMoodChange?.("idle");
      }, 550);
    }
    onClick?.();
  };

  return (
    <canvas
      ref={canvasRef}
      style={{ width: size, height: size }}
      className="cursor-pointer shrink-0 select-none"
      onMouseEnter={() => {
        hoverRef.current = true;
      }}
      onMouseLeave={() => {
        hoverRef.current = false;
      }}
      onClick={handleClick}
      title="Klik Nobi (klik 3x cepat untuk mode pusing!)"
    />
  );
};
