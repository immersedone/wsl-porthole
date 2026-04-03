import puppeteer from "puppeteer";
import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
const __dirname = path.dirname(fileURLToPath(import.meta.url));
const iconsDir = path.join(__dirname, "..", "src-tauri", "icons");
const svgPath = path.join(iconsDir, "icon.svg");

const sizes = [
  { size: 32, file: "32x32.png" },
  { size: 128, file: "128x128.png" },
  { size: 256, file: "128x128@2x.png" },
];

const icoSizes = [16, 24, 32, 48, 64, 128, 256];

async function main() {
  const svgContent = fs.readFileSync(svgPath, "utf-8");
  const browser = await puppeteer.launch({
    headless: true,
    args: ["--no-sandbox", "--disable-setuid-sandbox"],
  });

  const pngBuffers = {};

  // Generate all PNG sizes
  const allSizes = [...new Set([...sizes.map(s => s.size), ...icoSizes])];

  for (const size of allSizes) {
    const page = await browser.newPage();
    await page.setViewport({ width: size, height: size });
    await page.setContent(`
      <html>
        <body style="margin:0;padding:0;background:transparent;">
          <div style="width:${size}px;height:${size}px;">
            ${svgContent}
          </div>
        </body>
      </html>
    `);
    const buf = await page.screenshot({ type: "png", omitBackground: true });
    pngBuffers[size] = Buffer.from(buf);
    await page.close();
  }

  // Write the standard PNG files
  for (const { size, file } of sizes) {
    fs.writeFileSync(path.join(iconsDir, file), pngBuffers[size]);
    console.log(`  ${file} (${size}x${size})`);
  }

  // Build ICO file from PNG buffers
  const icoData = buildIco(icoSizes.map(s => pngBuffers[s]));
  fs.writeFileSync(path.join(iconsDir, "icon.ico"), icoData);
  console.log(`  icon.ico (${icoSizes.join(", ")}px)`);

  // Copy 128x128 as icns placeholder (real icns needs different tool)
  fs.writeFileSync(path.join(iconsDir, "icon.icns"), pngBuffers[128]);
  console.log(`  icon.icns (128x128 png placeholder)`);

  await browser.close();
  console.log("\nDone!");
}

function buildIco(pngBuffers) {
  const count = pngBuffers.length;
  // ICO header: reserved(2) + type(2) + count(2) = 6 bytes
  const headerSize = 6;
  const dirEntrySize = 16;
  const dirSize = dirEntrySize * count;

  let offset = headerSize + dirSize;
  const parts = [];

  // Header
  const header = Buffer.alloc(6);
  header.writeUInt16LE(0, 0);     // reserved
  header.writeUInt16LE(1, 2);     // type: ICO
  header.writeUInt16LE(count, 4); // count
  parts.push(header);

  // Directory entries
  const dirEntries = Buffer.alloc(dirSize);
  for (let i = 0; i < count; i++) {
    const buf = pngBuffers[i];
    // Parse width/height from PNG IHDR
    const w = buf.readUInt32BE(16);
    const h = buf.readUInt32BE(20);
    const off = i * 16;
    dirEntries.writeUInt8(w >= 256 ? 0 : w, off);      // width (0 = 256)
    dirEntries.writeUInt8(h >= 256 ? 0 : h, off + 1);   // height
    dirEntries.writeUInt8(0, off + 2);                    // color palette
    dirEntries.writeUInt8(0, off + 3);                    // reserved
    dirEntries.writeUInt16LE(1, off + 4);                 // color planes
    dirEntries.writeUInt16LE(32, off + 6);                // bits per pixel
    dirEntries.writeUInt32LE(buf.length, off + 8);        // size
    dirEntries.writeUInt32LE(offset, off + 12);           // offset
    offset += buf.length;
  }
  parts.push(dirEntries);

  // Image data
  for (const buf of pngBuffers) {
    parts.push(buf);
  }

  return Buffer.concat(parts);
}

main().catch(console.error);
