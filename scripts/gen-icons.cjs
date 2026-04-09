const sharp = require("sharp");
const fs = require("fs");
const path = require("path");

const ICONS = path.join(__dirname, "..", "src-tauri", "icons");
const ASSETS = path.join(__dirname, "..", "src", "assets");

// Same pixel data as BrandElfMascot.vue — ghost face mascot (dark theme, neutral)
const OUT = "#0f172a";   // outline (dark theme)
const bodyW = "#f8fafc"; // main face
const bodyL = "#ffffff"; // highlight
const bodyD = "#cbd5e1"; // shadow
const B = "#93c5fd";     // eye blue
const P_BLUSH = "#fca5a5"; // blush

// 16x16 grid matching BrandElfMascot.vue basePixels (neutral/default expression)
const basePixels = [
  "......OOOO......",
  "....OOLLLLO.....",
  "...OLLWWWWLOO...",
  "..OLWWWWWWWWWO..",
  ".OLWWWWWWWWWWWO.",
  ".OLWWWWWWWWWWWO.",
  ".OLWBEWWBWBEWWO.",
  ".OLWWWWWWWWWWWO.",
  ".OLWPWWWWWWWWPO.",
  ".OLWWWWWWWWWWWO.",
  "..ODWWWWWWWWWDO.",
  "...ODDDDDDDWDO..",
  "....OOOOOODWDO..",
  ".........ODDO...",
  ".........OO.....",
  "................"
];

// Build pixel list from grid
const pixels = [];
for (let y = 0; y < 16; y++) {
  for (let x = 0; x < 16; x++) {
    const ch = basePixels[y][x];
    let color = null;
    if (ch === 'O') color = OUT;
    else if (ch === 'W') color = bodyW;
    else if (ch === 'L') color = bodyL;
    else if (ch === 'D') color = bodyD;
    else if (ch === 'B') color = B;
    else if (ch === 'P') color = P_BLUSH;
    else if (ch === 'E') color = OUT;
    if (color) pixels.push([x, y, color]);
  }
}

// Grid is 16x16
const GRID_W = 16;
const GRID_H = 16;

function hexToRgba(hex) {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return { r, g, b, alpha: 255 };
}

async function renderPixelArt(scale) {
  const w = GRID_W * scale;
  const h = GRID_H * scale;
  // Create raw RGBA buffer
  const buf = Buffer.alloc(w * h * 4, 0); // all transparent
  for (const [x, y, col] of pixels) {
    const { r, g, b } = hexToRgba(col);
    for (let dy = 0; dy < scale; dy++) {
      for (let dx = 0; dx < scale; dx++) {
        const px = x * scale + dx;
        const py = y * scale + dy;
        if (px < w && py < h) {
          const off = (py * w + px) * 4;
          buf[off] = r;
          buf[off + 1] = g;
          buf[off + 2] = b;
          buf[off + 3] = 255;
        }
      }
    }
  }
  return sharp(buf, { raw: { width: w, height: h, channels: 4 } })
    .png()
    .toBuffer();
}

async function main() {
  // Render at high quality scale for source
  const base128 = await renderPixelArt(8); // 16*8 = 128
  const base64 = await renderPixelArt(4);  // 16*4 = 64

  const sizes = [
    ["32x32.png", 32],
    ["64x64.png", 64],
    ["128x128.png", 128],
    ["128x128@2x.png", 256],
    ["256x256.png", 256],
    ["icon.png", 512],
    ["Square30x30Logo.png", 30],
    ["Square44x44Logo.png", 44],
    ["Square71x71Logo.png", 71],
    ["Square89x89Logo.png", 89],
    ["Square107x107Logo.png", 107],
    ["Square142x142Logo.png", 142],
    ["Square150x150Logo.png", 150],
    ["Square284x284Logo.png", 284],
    ["Square310x310Logo.png", 310],
    ["StoreLogo.png", 50],
  ];

  for (const [file, sz] of sizes) {
    const src = sz >= 128 ? base128 : base64;
    fs.writeFileSync(
      path.join(ICONS, file),
      await sharp(src)
        .resize(sz, sz, { kernel: "nearest" })
        .png()
        .toBuffer()
    );
    console.log(file, sz + "x" + sz);
  }

  // favicon
  const faviconBuf = await renderPixelArt(2); // 16*2 = 32
  fs.writeFileSync(
    path.join(ASSETS, "favicon.png"),
    await sharp(faviconBuf).resize(32, 32, { kernel: "nearest" }).png().toBuffer()
  );
  console.log("favicon.png");

  // multi-size ICO
  const icoSizes = [16, 24, 32, 48, 64, 128, 256];
  const entries = [];
  for (const sz of icoSizes) {
    const src = sz >= 128 ? base128 : base64;
    entries.push({
      size: sz,
      data: await sharp(src).resize(sz, sz, { kernel: "nearest" }).png().toBuffer(),
    });
  }

  const hdrSize = 6 + entries.length * 16;
  let total = hdrSize;
  for (const e of entries) total += e.data.length;
  const ico = Buffer.alloc(total);
  let o = 0;
  ico.writeUInt16LE(0, o); o += 2;
  ico.writeUInt16LE(1, o); o += 2;
  ico.writeUInt16LE(entries.length, o); o += 2;

  let dataOff = hdrSize;
  for (const e of entries) {
    const w = e.size >= 256 ? 0 : e.size;
    const h = e.size >= 256 ? 0 : e.size;
    ico.writeUInt8(w, o); o++;
    ico.writeUInt8(h, o); o++;
    ico.writeUInt8(0, o); o++;
    ico.writeUInt8(0, o); o++;
    ico.writeUInt16LE(1, o); o += 2;
    ico.writeUInt16LE(32, o); o += 2;
    ico.writeUInt32LE(e.data.length, o); o += 4;
    ico.writeUInt32LE(dataOff, o); o += 4;
    e.data.copy(ico, dataOff);
    dataOff += e.data.length;
  }
  fs.writeFileSync(path.join(ICONS, "icon.ico"), ico);
  console.log("icon.ico", entries.length, "sizes");

  console.log("All starburst icons generated!");
}

main().catch((e) => { console.error(e); process.exit(1); });
