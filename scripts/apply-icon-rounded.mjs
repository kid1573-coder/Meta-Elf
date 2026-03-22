/**
 * Applies a rounded-rectangle mask to the master icon (mainstream app / squircle-style corners).
 * Reads src-tauri/icon-source.png, writes src-tauri/.icon-for-tauri-build.png for `tauri icon`.
 */
import sharp from "sharp";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, "..");
const input = join(root, "src-tauri", "icon-source.png");
const output = join(root, "src-tauri", ".icon-for-tauri-build.png");

/** Output square size (Tauri recommends ~1024 for source). */
const SIZE = 1024;
/** Corner radius as fraction of side length (~iOS app icon proportion). */
const CORNER_RATIO = 0.2237;

const rx = Math.round(SIZE * CORNER_RATIO);
const roundedMask = Buffer.from(
  `<svg width="${SIZE}" height="${SIZE}" xmlns="http://www.w3.org/2000/svg">
    <rect width="${SIZE}" height="${SIZE}" rx="${rx}" ry="${rx}" fill="white"/>
  </svg>`
);

await sharp(input)
  .resize(SIZE, SIZE, { fit: "cover", position: "centre" })
  .composite([{ input: roundedMask, blend: "dest-in" }])
  .png()
  .toFile(output);

console.log("Rounded icon for Tauri:", output);
