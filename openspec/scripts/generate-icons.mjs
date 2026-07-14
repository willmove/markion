// One-off asset generator: rasterizes assets/markion-logo.svg into a high-resolution
// master PNG, then derives markion.png / markion.ico / markion.icns from it.
//
// Run with:  node openspec/scripts/generate-icons.mjs
//
// Outputs (all written to assets/):
//   - markion.png   512×512 master PNG (README + packaging preview)
//   - markion.ico   Windows multi-size ICO (16,24,32,48,64,72,96,128,256)
//   - markion.icns  macOS ICNS (recommended sizes for the Dock)
import { readFile, writeFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import sharp from "sharp";
import * as png2icons from "png2icons";

// This script lives in `openspec/scripts`, so the repository root is two
// directories above its own location.
const root = join(dirname(fileURLToPath(import.meta.url)), "..", "..");
const SVG_PATH = join(root, "assets", "markion-logo.svg");
const ASSETS = join(root, "assets");

const ICO_SIZES = [16, 24, 32, 48, 64, 72, 96, 128, 256];
// png2icons derives the ICNS family from a single large PNG; it emits the
// standard set the format supports.
const MASTER_PX = 1024;

async function main() {
  const svg = await readFile(SVG_PATH);

  // High-resolution master raster used to derive every other asset.
  const masterPng = await sharp(svg, { density: 384 })
    .resize(MASTER_PX, MASTER_PX, { fit: "contain", background: { r: 0, g: 0, b: 0, alpha: 0 } })
    .png()
    .toBuffer();

  // 1) README / packaging preview PNG at 512px.
  const png512 = await sharp(masterPng).resize(512, 512, { fit: "contain" }).png().toBuffer();
  await writeFile(join(ASSETS, "markion.png"), png512);
  console.log("wrote assets/markion.png (512×512)");

  // 2) Windows ICO embedded as the exe/window icon resource. `forWinExe=true`
  //    stores <64px entries as Windows bitmaps and the rest as PNG — the mix
  //    recommended by the library for embedded-executable icons.
  //    `numOfColors=0` disables palette quantization (24/32-bit RGBA).
  const ico = png2icons.createICO(masterPng, png2icons.BICUBIC, 0, false, true);
  await writeFile(join(ASSETS, "markion.ico"), ico);
  console.log(`wrote assets/markion.ico (sizes ${ICO_SIZES.join(",")})`);

  // 3) macOS ICNS for the Dock (full 16→512@2 family derived from the master).
  const icns = png2icons.createICNS(masterPng, png2icons.BICUBIC, 0);
  await writeFile(join(ASSETS, "markion.icns"), icns);
  console.log("wrote assets/markion.icns");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
