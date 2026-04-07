const sharp = require("sharp");
const fs = require("fs");
const path = require("path");

const ICONS = path.join(__dirname, "..", "src-tauri", "icons");
const ASSETS = path.join(__dirname, "..", "src", "assets");

// Same pixel data as BrandElfMascot.vue — Claude Code starburst
const C  = "#e97451"; // coral main
const CL = "#f09070"; // coral light
const CD = "#c4533a"; // coral dark
const CK = "#a83820"; // coral deep
const W  = "#ffcc66"; // warm center
const WL = "#ffe0a0"; // warm highlight
const T  = "#ff8c42"; // orange tip

const pixels = [
  // top ray
  [7,0,T],[8,0,T],
  [6,1,CL],[7,1,T],[8,1,T],[9,1,CL],
  [7,2,C],[8,2,C],
  // upper left ray
  [3,2,T],[4,2,CL],
  [2,3,T],[3,3,CL],[4,3,C],
  [3,4,C],[4,4,C],
  // upper right ray
  [11,2,CL],[12,2,T],
  [11,3,C],[12,3,CL],[13,3,T],
  [11,4,C],[12,4,C],
  // left ray
  [1,7,T],[1,8,T],
  [2,6,CL],[2,7,C],[2,8,C],[2,9,CL],
  [3,7,C],[3,8,C],
  // right ray
  [13,7,T],[13,8,T],
  [13,6,CL],[13,7,C],[13,8,C],[13,9,CL],
  [12,7,C],[12,8,C],
  // lower left ray
  [3,11,C],[4,11,C],
  [2,10,CL],[3,10,C],[4,10,C],
  [3,12,CL],[4,12,T],
  [3,13,T],
  // lower right ray
  [11,11,C],[12,11,C],
  [11,10,C],[12,10,CL],[13,10,T],
  [11,12,C],[12,12,CL],
  [12,13,T],
  // bottom ray
  [7,14,T],[8,14,T],
  [6,13,CL],[7,13,C],[8,13,C],[9,13,CL],
  [7,12,C],[8,12,C],
  // inner body
  [4,4,C],[5,4,C],[6,4,C],[7,4,C],[8,4,C],[9,4,C],[10,4,C],[11,4,C],
  [3,5,C],[4,5,C],[5,5,C],[6,5,C],[7,5,C],[8,5,C],[9,5,C],[10,5,C],[11,5,C],[12,5,C],
  [3,6,C],[4,6,C],[5,6,CD],[6,6,CD],[7,6,CD],[8,6,CD],[9,6,CD],[10,6,CD],[11,6,C],[12,6,C],
  [3,7,CD],[4,7,CD],[5,7,CD],[6,7,W],[7,7,W],[8,7,W],[9,7,W],[10,7,CD],[11,7,CD],[12,7,CD],
  [3,8,CD],[4,8,CD],[5,8,W],[6,8,W],[7,8,WL],[8,8,WL],[9,8,W],[10,8,W],[11,8,CD],[12,8,CD],
  [3,9,CD],[4,9,CD],[5,9,CD],[6,9,W],[7,9,W],[8,9,W],[9,9,W],[10,9,CD],[11,9,CD],[12,9,CD],
  [3,10,C],[4,10,C],[5,10,CD],[6,10,CD],[7,10,CD],[8,10,CD],[9,10,CD],[10,10,CD],[11,10,C],[12,10,C],
  [4,11,C],[5,11,C],[6,11,C],[7,11,C],[8,11,C],[9,11,C],[10,11,C],[11,11,C],
  // eye highlights
  [6,7,CK],[7,6,CK],
  [8,7,CK],[9,6,CK],
  // corner sparkles
  [0,0,W],[14,0,W],[0,14,W],[14,14,W],
  [1,1,WL],[13,1,WL],[1,13,WL],[13,13,WL],
];

// Grid is 15x15
const GRID_W = 15;
const GRID_H = 15;

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
  const base128 = await renderPixelArt(8); // 15*8 = 120, padded with transparency
  const base64 = await renderPixelArt(4);  // 15*4 = 60

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
  const faviconBuf = await renderPixelArt(2); // 15*2 = 30
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
