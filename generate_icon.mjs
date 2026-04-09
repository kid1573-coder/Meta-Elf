import { createCanvas } from 'canvas';
import fs from 'fs';

const GRID = 16;
const CANVAS_SIZE = 512;
const SCALE = Math.floor(CANVAS_SIZE / GRID); // 32

const OUT = '#1e1b4b';
const W = '#f8fafc';
const L = '#ffffff';
const D = '#cbd5e1';
const B = '#93c5fd';
const E = '#1e1b4b';
const P = '#fca5a5';

const colors = {
  '.': null,
  'O': OUT,
  'W': W,
  'L': L,
  'D': D,
  'B': B,
  'E': E,
  'P': P
};

const pixels = [
  "......OOOO......",
  "....OOLLLLO.....",
  "...OLLWWWWLOO...",
  "..OLWWWWWWWWWO..",
  ".OLWWWWWWWWWWWO.",
  ".OLWWWWWWWWWWWO.",
  ".OLWBEWWBWBEWWO.",
  ".OLWWWWWWWWWWWO.",
  ".OLWPWWWWWWWPWO.",
  ".OLWWWWWWWWWWWO.",
  "..ODWWWWWWWWWDO.",
  "...ODDDDDDDWDO..",
  "....OOOOOODWDO..",
  ".........ODDO...",
  ".........OO.....",
  "................"
];

const canvas = createCanvas(CANVAS_SIZE, CANVAS_SIZE);
const ctx = canvas.getContext('2d');

// Transparent background
ctx.clearRect(0, 0, CANVAS_SIZE, CANVAS_SIZE);

// Since SCALE is exactly 32 and GRID is 16, 16*32 = 512. It perfectly fills the canvas.
for (let y = 0; y < 16; y++) {
  for (let x = 0; x < 16; x++) {
    const char = pixels[y][x];
    const color = colors[char];
    if (color) {
      ctx.fillStyle = color;
      ctx.fillRect(x * SCALE, y * SCALE, SCALE, SCALE);
    }
  }
}

const buffer = canvas.toBuffer('image/png');
fs.writeFileSync('icon.png', buffer);
console.log('icon.png created with Ghost mascot (16x16 perfectly scaled to 512).');
