const VERSION_SPECS: Array<{
  version: number;
  size: number;
  dataCodewords: number;
  ecCodewords: number;
  alignCenters: number[];
}> = [
  { version: 1, size: 21, dataCodewords: 19, ecCodewords: 7, alignCenters: [] },
  { version: 2, size: 25, dataCodewords: 34, ecCodewords: 10, alignCenters: [6, 18] },
  { version: 3, size: 29, dataCodewords: 55, ecCodewords: 15, alignCenters: [6, 22] },
  { version: 4, size: 33, dataCodewords: 80, ecCodewords: 20, alignCenters: [6, 26] },
  { version: 5, size: 37, dataCodewords: 108, ecCodewords: 26, alignCenters: [6, 30] },
  { version: 6, size: 41, dataCodewords: 136, ecCodewords: 18, alignCenters: [6, 34] },
];

const GF_EXP = new Uint8Array(512);
const GF_LOG = new Uint8Array(256);
(() => {
  let x = 1;
  for (let i = 0; i < 255; i++) {
    GF_EXP[i] = x;
    GF_LOG[x] = i;
    x <<= 1;
    if (x & 0x100) x ^= 0x11d;
  }
  for (let i = 255; i < 512; i++) {
    GF_EXP[i] = GF_EXP[i - 255];
  }
})();

function gfMul(a: number, b: number): number {
  if (a === 0 || b === 0) return 0;
  return GF_EXP[GF_LOG[a] + GF_LOG[b]];
}

function rsGeneratorPoly(degree: number): Uint8Array {
  let poly = new Uint8Array([1]);
  for (let i = 0; i < degree; i++) {
    const next = new Uint8Array(poly.length + 1);
    const factor = GF_EXP[i];
    for (let j = 0; j < poly.length; j++) {
      next[j] ^= poly[j];
      next[j + 1] ^= gfMul(poly[j], factor);
    }
    poly = next;
  }
  return poly;
}

function rsRemainder(data: Uint8Array, ecCount: number): Uint8Array {
  const gen = rsGeneratorPoly(ecCount);
  const rem = new Uint8Array(ecCount);
  for (let i = 0; i < data.length; i++) {
    const factor = data[i] ^ rem[0];
    rem.copyWithin(0, 1);
    rem[ecCount - 1] = 0;
    if (factor !== 0) {
      for (let j = 0; j < ecCount; j++) {
        rem[j] ^= gfMul(gen[j + 1], factor);
      }
    }
  }
  return rem;
}

function formatBitsLevelLMask0(): number {
  return 0x77c4;
}

export function generateQrMatrix(text: string): boolean[][] {
  const bytes = new TextEncoder().encode(text);
  const spec =
    VERSION_SPECS.find((v) => bytes.length + 2 <= v.dataCodewords) ??
    VERSION_SPECS[VERSION_SPECS.length - 1];

  const payload = bytes.slice(0, spec.dataCodewords - 2);
  const bits: number[] = [];
  const pushBits = (val: number, len: number) => {
    for (let i = len - 1; i >= 0; i--) {
      bits.push((val >> i) & 1);
    }
  };

  pushBits(0b0100, 4);
  pushBits(payload.length, 8);
  for (const b of payload) pushBits(b, 8);

  const totalDataBits = spec.dataCodewords * 8;
  const term = Math.min(4, totalDataBits - bits.length);
  for (let i = 0; i < term; i++) bits.push(0);
  while (bits.length % 8 !== 0) bits.push(0);

  const dataBytes = new Uint8Array(spec.dataCodewords);
  let byteIdx = 0;
  for (let i = 0; i < bits.length; i += 8) {
    let v = 0;
    for (let j = 0; j < 8; j++) v = (v << 1) | bits[i + j];
    dataBytes[byteIdx++] = v;
  }
  const padBytes = [0xec, 0x11];
  let padIdx = 0;
  while (byteIdx < spec.dataCodewords) {
    dataBytes[byteIdx++] = padBytes[padIdx++ % 2];
  }

  // Interleave blocks
  let codewords: Uint8Array;
  if (spec.version === 6) {
    const b1 = dataBytes.slice(0, 68);
    const b2 = dataBytes.slice(68, 136);
    const ec1 = rsRemainder(b1, 18);
    const ec2 = rsRemainder(b2, 18);
    codewords = new Uint8Array(136 + 36);
    let k = 0;
    for (let i = 0; i < 68; i++) {
      codewords[k++] = b1[i];
      codewords[k++] = b2[i];
    }
    for (let i = 0; i < 18; i++) {
      codewords[k++] = ec1[i];
      codewords[k++] = ec2[i];
    }
  } else {
    const ec = rsRemainder(dataBytes, spec.ecCodewords);
    codewords = new Uint8Array(dataBytes.length + ec.length);
    codewords.set(dataBytes, 0);
    codewords.set(ec, dataBytes.length);
  }

  const size = spec.size;
  const mat: boolean[][] = Array.from({ length: size }, () => Array(size).fill(false));
  const isFunc: boolean[][] = Array.from({ length: size }, () => Array(size).fill(false));

  const setFunc = (r: number, c: number, dark: boolean) => {
    if (r < 0 || r >= size || c < 0 || c >= size) return;
    mat[r][c] = dark;
    isFunc[r][c] = true;
  };

  const drawFinder = (top: number, left: number) => {
    for (let dr = -1; dr <= 7; dr++) {
      for (let dc = -1; dc <= 7; dc++) {
        const r = top + dr;
        const c = left + dc;
        if (r < 0 || r >= size || c < 0 || c >= size) continue;
        const inOuter = dr >= 0 && dr <= 6 && dc >= 0 && dc <= 6;
        const dark =
          inOuter &&
          (dr === 0 ||
            dr === 6 ||
            dc === 0 ||
            dc === 6 ||
            (dr >= 2 && dr <= 4 && dc >= 2 && dc <= 4));
        setFunc(r, c, dark);
      }
    }
  };

  drawFinder(0, 0);
  drawFinder(0, size - 7);
  drawFinder(size - 7, 0);

  // Timing patterns
  for (let i = 8; i < size - 8; i++) {
    setFunc(6, i, i % 2 === 0);
    setFunc(i, 6, i % 2 === 0);
  }

  // Alignment pattern(s)
  for (const r of spec.alignCenters) {
    for (const c of spec.alignCenters) {
      if (isFunc[r][c]) continue;
      for (let dr = -2; dr <= 2; dr++) {
        for (let dc = -2; dc <= 2; dc++) {
          const dark = Math.max(Math.abs(dr), Math.abs(dc)) !== 1;
          setFunc(r + dr, c + dc, dark);
        }
      }
    }
  }

  // Format info
  const fmt = formatBitsLevelLMask0();
  for (let i = 0; i <= 5; i++) setFunc(8, i, ((fmt >> (14 - i)) & 1) === 1);
  setFunc(8, 7, ((fmt >> 8) & 1) === 1);
  setFunc(8, 8, ((fmt >> 7) & 1) === 1);
  setFunc(7, 8, ((fmt >> 6) & 1) === 1);
  for (let i = 0; i < 6; i++) setFunc(5 - i, 8, ((fmt >> (5 - i)) & 1) === 1);

  for (let i = 0; i < 7; i++) setFunc(size - 1 - i, 8, ((fmt >> (14 - i)) & 1) === 1);
  setFunc(size - 8, 8, true);
  for (let i = 0; i < 8; i++) setFunc(8, size - 8 + i, ((fmt >> (7 - i)) & 1) === 1);

  // Data placement
  let bitIndex = 0;
  const totalBits = codewords.length * 8;
  let upward = true;
  for (let col = size - 1; col >= 1; col -= 2) {
    if (col === 6) col = 5;
    for (let step = 0; step < size; step++) {
      const r = upward ? size - 1 - step : step;
      for (const dc of [0, -1]) {
        const c = col + dc;
        if (isFunc[r][c]) continue;
        let dark = false;
        if (bitIndex < totalBits) {
          const cw = codewords[bitIndex >> 3];
          dark = ((cw >> (7 - (bitIndex & 7))) & 1) === 1;
          bitIndex++;
        }
        if ((r + c) % 2 === 0) dark = !dark;
        mat[r][c] = dark;
      }
    }
    upward = !upward;
  }

  return mat;
}
