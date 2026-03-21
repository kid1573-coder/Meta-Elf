/**
 * Generates minimal solid-color PNGs for Tauri bundle (no extra deps).
 */
import { deflateSync } from "node:zlib";
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const iconsDir = join(__dirname, "..", "src-tauri", "icons");

function crc32(buf) {
  let c = 0xffffffff;
  for (let i = 0; i < buf.length; i++) {
    c ^= buf[i];
    for (let j = 0; j < 8; j++) {
      c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    }
  }
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type, data) {
  const len = Buffer.alloc(4);
  len.writeUInt32BE(data.length, 0);
  const typeBuf = Buffer.from(type, "ascii");
  const crcBuf = Buffer.alloc(4);
  const crc = crc32(Buffer.concat([typeBuf, data]));
  crcBuf.writeUInt32BE(crc, 0);
  return Buffer.concat([len, typeBuf, data, crcBuf]);
}

function solidPng(w, h, r, g, b) {
  const bpp = 3;
  const row = 1 + w * bpp;
  const raw = Buffer.alloc(row * h);
  for (let y = 0; y < h; y++) {
    const o = y * row;
    raw[o] = 0;
    for (let x = 0; x < w; x++) {
      const i = o + 1 + x * bpp;
      raw[i] = r;
      raw[i + 1] = g;
      raw[i + 2] = b;
    }
  }
  const idat = deflateSync(raw);
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(w, 0);
  ihdr.writeUInt32BE(h, 4);
  ihdr[8] = 8;
  ihdr[9] = 2;
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;
  const sig = Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]);
  const iend = chunk("IEND", Buffer.alloc(0));
  return Buffer.concat([sig, chunk("IHDR", ihdr), chunk("IDAT", idat), iend]);
}

mkdirSync(iconsDir, { recursive: true });
const purple = [88, 86, 214];
const png32 = solidPng(32, 32, ...purple);
const png128 = solidPng(128, 128, ...purple);
const png256 = solidPng(256, 256, ...purple);
writeFileSync(join(iconsDir, "32x32.png"), png32);
writeFileSync(join(iconsDir, "128x128.png"), png128);
writeFileSync(join(iconsDir, "128x128@2x.png"), png256);
writeFileSync(join(iconsDir, "icon.png"), png128);
console.log("Wrote PNG icons to", iconsDir);
console.log("Run: npm run tauri icon src-tauri/icons/icon.png");
for (const f of ["icon.icns", "icon.ico"]) {
  console.log("Note: generate", f, "with `npm run tauri icon` after installing Rust.");
}
